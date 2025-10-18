mod cli;
use clap::error::ErrorKind;
use clap::ArgMatches;
use clap_complete::{generate_to, Shell};

use std::{process::ExitCode, str::FromStr};

use crate::cli::otel::{capture_telemetry, init_tracer};
use crate::cli::pact_broker_docker;
use crate::cli::pact_broker_ruby;

pub fn main() -> ExitCode {
    let app = cli::build_cli();
    let matches = app.clone().try_get_matches();

    let (
        enable_otel,
        enable_otel_logs,
        otel_exporter,
        otel_exporter_endpoint,
        otel_exporter_protocol,
        log_level,
    ) = match &matches {
        Ok(m) => (
            m.get_flag("enable-otel"),
            m.get_flag("enable-otel-logs"),
            m.get_one::<String>("otel-exporter"),
            m.get_one::<String>("otel-exporter-endpoint"),
            m.get_one::<String>("otel-exporter-protocol"),
            m.get_one::<String>("log-level")
                .and_then(|lvl| lvl.parse::<tracing::Level>().ok()),
        ),
        Err(_) => (false, false, None, None, None, None),
    };
    let otel_config = if enable_otel {
        Some(crate::cli::otel::OtelConfig {
            exporter: otel_exporter.cloned(),
            endpoint: otel_exporter_endpoint.cloned(),
            protocol: otel_exporter_protocol.cloned(),
        })
    } else {
        None
    };
    let mut tracer_provider = None;
    let mut log_provider = None;
    let rt: tokio::runtime::Runtime =
        tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async {
        tracer_provider = if enable_otel {
            Some(init_tracer(otel_config.unwrap()))
        } else {
            None
        };
        log_provider = if enable_otel_logs {
            Some(crate::cli::otel::init_logs(log_level)).unwrap()
        } else {
            None
        };
    });

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
                            Ok(_) => {
                                capture_telemetry(&std::env::args().collect::<Vec<_>>(), 0, None);
                                ExitCode::SUCCESS
                            }
                            Err(code) => {
                                capture_telemetry(&std::env::args().collect::<Vec<_>>(), 1, None);
                                code
                            }
                        };
                        // return Ok(());
                    }
                    Some("ruby") => {
                        let standalone_args = args.subcommand_matches("ruby").unwrap();
                        let res = pact_broker_ruby::run(standalone_args);
                        return match res {
                            Ok(_) => {
                                capture_telemetry(&std::env::args().collect::<Vec<_>>(), 0, None);
                                ExitCode::SUCCESS
                            }
                            Err(err) => {
                                println!("{}", err);
                                capture_telemetry(
                                    &std::env::args().collect::<Vec<_>>(),
                                    1,
                                    Some(err.as_str()),
                                );
                                ExitCode::from(1)
                            }
                        };
                    }
                    _ => {}
                }

                let raw_args: Vec<String> = std::env::args().collect();
                let matches_result = Ok(args.clone());
                match pact_broker_cli::handle_matches(&matches_result, Some(raw_args)) {
                    Ok(()) => {
                        capture_telemetry(&std::env::args().collect::<Vec<_>>(), 0, None);
                        Ok(())
                    }
                    Err(e) => {
                        capture_telemetry(&std::env::args().collect::<Vec<_>>(), 1, None);
                        Err(e)
                    }
                }
            }
            Some(("pactflow", args)) => {
                match pact_broker_cli::cli::pactflow_client::run(args, std::env::args().collect()) {
                    Ok(_) => {
                        capture_telemetry(&std::env::args().collect::<Vec<_>>(), 0, None);
                        Ok(())
                    }
                    Err(error) => {
                        capture_telemetry(&std::env::args().collect::<Vec<_>>(), error, None);
                        Err(ExitCode::from(error as u8))
                    }
                }
            }
            Some(("stub", args)) => {
                let res = pact_stub_server_cli::process_stub_command(args);
                capture_telemetry(&std::env::args().collect::<Vec<_>>(), 0, None);
                res
            }
            Some(("completions", args)) => {
                let res = generate_completions(args);
                capture_telemetry(&std::env::args().collect::<Vec<_>>(), 0, None);
                res
            }
            Some(("plugin", args)) => {
                let res = pact_plugin_cli::process_plugin_command(args);
                capture_telemetry(&std::env::args().collect::<Vec<_>>(), 0, None);
                res
            }
            Some(("mock", args)) => {
                let res = pact_mock_server_cli::process_mock_command(args);
                capture_telemetry(&std::env::args().collect::<Vec<_>>(), 0, None);
                res
            }
            Some(("verifier", args)) => {
                let res = pact_verifier_cli::process_verifier_command(args);
                capture_telemetry(&std::env::args().collect::<Vec<_>>(), 0, None);
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

            _ => {
                capture_telemetry(
                    &std::env::args().collect::<Vec<_>>(),
                    err.exit_code(),
                    Some(&err.to_string()),
                );

                err.exit()
            }
        },
    };
    if let Some(tracer_provider) = tracer_provider {
        if enable_otel {
            let _ = tracer_provider.shutdown();
        }
    }
    if let Some(log_provider) = log_provider {
        if enable_otel_logs {
            let _ = log_provider.shutdown();
        }
    }
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
