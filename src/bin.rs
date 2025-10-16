mod cli;
use clap::error::ErrorKind;
use clap::ArgMatches;
use clap_complete::{generate_to, Shell};

use std::{process::ExitCode, str::FromStr};

use crate::cli::pact_broker_docker;
use crate::cli::pact_broker_ruby;

pub fn main() -> Result<(), ExitCode> {
    let app = cli::build_cli();
    let cloned_app = app.clone();
    match app.clone().try_get_matches() {
        Ok(results) => match results.subcommand() {
            Some(("broker", args)) | Some(("pactflow", args)) => {
                // if args subcommand is docker or standalone, offset to those subcommands
                let subcommand = args.subcommand_name();
                match subcommand {
                    Some("docker") => {
                        let docker_args = args.subcommand_matches("docker").unwrap();
                        return pact_broker_docker::run(docker_args);
                        // return Ok(());
                    }
                    Some("ruby") => {
                        let standalone_args = args.subcommand_matches("ruby").unwrap();
                        pact_broker_ruby::run(standalone_args);
                        return Ok(());
                    }
                    _ => {}
                }

                let raw_args: Vec<String> = std::env::args().collect();
                let matches_result = Ok(args.clone());
                pact_broker_cli::handle_matches(&matches_result, Some(raw_args))
            }
            Some(("stub", args)) => pact_stub_server_cli::process_stub_command(args),
            Some(("completions", args)) => generate_completions(args),
            Some(("plugin", args)) => pact_plugin_cli::process_plugin_command(args),
            Some(("mock", args)) => pact_mock_server_cli::process_mock_command(args),
            Some(("verifier", args)) => pact_verifier_cli::process_verifier_command(args),
            _ => {
                cli::build_cli().print_help().unwrap();
                Ok(())
            }
        },

        Err(err) => match err.kind() {
            ErrorKind::DisplayHelp => {
                // let _ = err.print();
                err.exit();
            }
            ErrorKind::DisplayVersion => {
                let error_message = err.render().to_string();
                let versions = [
                    (
                        "pact-verifier",
                        pact_verifier_cli::print_version as fn(),
                    ),
                    ("pact-mock", pact_mock_server_cli::print_version as fn()),
                    ("pact-stub", pact_stub_server_cli::print_version as fn()),
                ];
                for (name, print_fn) in &versions {
                    if error_message.contains(name) {
                        print_fn();
                        println!();
                        return Ok(());
                    }
                }
                // let _ = err.print();
                err.exit();
            }
            _ => err.exit(),
        },
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
