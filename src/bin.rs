mod cli;
use clap::error::ErrorKind;
use clap::ArgMatches;
use clap_complete::{generate_to, Shell};

use std::{process::ExitCode, str::FromStr};

use crate::cli::pact_broker_docker;
use crate::cli::pact_broker_ruby;

pub fn main() -> ExitCode {
    let app = cli::build_cli();
    let res = match app.clone().try_get_matches() {
        Ok(results) => match results.subcommand() {
            Some(("broker", args)) => {
                // if args subcommand is docker or standalone, offset to those subcommands
                let subcommand = args.subcommand_name();
                match subcommand {
                    Some("docker") => {
                        let docker_args = args.subcommand_matches("docker").unwrap();
                        return match pact_broker_docker::run(docker_args) {
                            Ok(_) => ExitCode::SUCCESS,
                            Err(code) => code,
                        };
                        // return Ok(());
                    }
                    Some("ruby") => {
                        let standalone_args = args.subcommand_matches("ruby").unwrap();
                        let res = pact_broker_ruby::run(standalone_args);
                        match res {
                            Ok(_) => return ExitCode::SUCCESS,
                            Err(err) => {
                                return {
                                    eprintln!("{}", err);
                                    ExitCode::from(1)
                                }
                            }
                        }
                    }
                    _ => {}
                }

                let raw_args: Vec<String> = std::env::args().collect();
                let matches_result = Ok(args.clone());
                match pact_broker_cli::handle_matches(&matches_result, Some(raw_args)) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(e),
                }
            }
            Some(("pactflow", args)) => {
                match pact_broker_cli::cli::pactflow_client::run(args, std::env::args().collect()) {
                    Ok(_) => Ok(()),
                    Err(error) => Err(ExitCode::from(error as u8)),
                }
            }
            Some(("stub", args)) => {
                let res = pact_stub_server_cli::process_stub_command(args);
                res
            }
            Some(("completions", args)) => {
                let res = generate_completions(args);
                res
            }
            Some(("plugin", args)) => {
                let res = pact_plugin_cli::process_plugin_command(args);
                res
            }
            Some(("mock", args)) => {
                let res = pact_mock_server_cli::process_mock_command(args);
                res
            }
            Some(("verifier", args)) => {
                let res = pact_verifier_cli::process_verifier_command(args);
                res
            }
            _ => {
                cli::build_cli().print_help().unwrap();
                Ok(())
            }
        },

        Err(err) => match err.kind() {
            ErrorKind::DisplayHelp => {
                err.exit();
            }
            ErrorKind::DisplayVersion => {
                let error_message = err.render().to_string();
                let versions = [
                    ("pact-verifier", pact_verifier_cli::print_version as fn()),
                    ("pact-mock", pact_mock_server_cli::print_version as fn()),
                    ("pact-stub", pact_stub_server_cli::print_version as fn()),
                ];
                for (name, print_fn) in &versions {
                    if error_message.contains(name) {
                        print_fn();
                        println!();
                        return ExitCode::SUCCESS;
                    }
                }
                err.exit();
            }
            _ => err.exit(),
        },
    };
    match res {
        Ok(_) => ExitCode::SUCCESS,
        Err(code) => code,
    }
}

fn generate_completions(args: &ArgMatches) -> Result<(), ExitCode> {
    let shell = match args.get_one::<String>("shell") {
        Some(shell) => shell,
        None => {
            eprintln!("Error: a shell is required");
            return Err(ExitCode::from(1));
        }
    };
    let out_dir = match args.get_one::<String>("dir") {
        Some(dir) => dir.to_string(),
        None => {
            eprintln!("Error: a directory is expected");
            return Err(ExitCode::from(1));
        }
    };
    let mut cmd = cli::build_cli();
    let shell_enum = match Shell::from_str(shell) {
        Ok(shell_enum) => shell_enum,
        Err(_) => {
            eprintln!("Error: invalid shell '{}'", shell);
            return Err(ExitCode::from(2));
        }
    };
    match generate_to(shell_enum, &mut cmd, "pact".to_string(), &out_dir) {
        Ok(path) => {
            println!(
                "ℹ️  {} shell completions for pact written to {}",
                shell_enum,
                path.display()
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("Error generating completions: {}", e);
            Err(ExitCode::from(3))
        }
    }
}
