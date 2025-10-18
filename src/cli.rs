use clap::{Arg, Command, CommandFactory};

use crate::cli::{
    pact_broker_docker::add_docker_broker_subcommand, pact_broker_ruby::add_ruby_broker_subcommand,
};

pub mod otel;
pub mod pact_broker_docker;
pub mod pact_broker_ruby;

pub fn build_cli() -> Command {
    let app = Command::new("pact")
        .about("Pact consolidated CLI - pact_core_mock_server, pact_verifier, pact-stub-server, pact-plugin-cli, pact-broker-cli in a single binary")
        .args(add_otel_options_args())
        .subcommand(
            pact_broker_cli::cli::pact_broker_client::add_pact_broker_client_command()
            .name("broker")
            .subcommand(add_ruby_broker_subcommand())
            .subcommand(add_docker_broker_subcommand())
        )
        .args(pact_broker_cli::cli::add_logging_arguments())
        .subcommand(
            pact_broker_cli::cli::pactflow_client::add_pactflow_client_command().name("pactflow"),
        )
        .subcommand(add_completions_subcommand())
        .subcommand(pact_plugin_cli::Cli::command().name("plugin"))
        .subcommand(pact_mock_server_cli::setup_args().name("mock"))
        .subcommand(pact_verifier_cli::args::setup_app().name("verifier"))
        .subcommand(pact_stub_server_cli::build_args().name("stub"));
    app
}

fn add_completions_subcommand() -> Command {
    Command::new("completions") 
    .about("Generates completion scripts for your shell")
    .arg(Arg::new("shell")
        .value_name("SHELL")
        .required(true)
        .value_parser(clap::builder::PossibleValuesParser::new(&["bash", "fish", "zsh", "powershell", "elvish"]))
        .help("The shell to generate the script for"))
    .arg(Arg::new("dir")
        .short('d')
        .long("dir")
        .value_name("DIRECTORY")
        .required(false)
        .default_value(".")
        .num_args(1)
        .value_parser(clap::builder::NonEmptyStringValueParser::new())
        .help("The directory to write the shell completions to, default is the current directory"))
}

fn add_otel_options_args() -> Vec<Arg> {
    vec![
        Arg::new("enable-otel")
            .long("enable-otel")
            .help("Enable OpenTelemetry tracing")
            .global(true)
            .hide(true)
            .action(clap::ArgAction::SetTrue),
        Arg::new("enable-otel-logs")
            .long("enable-otel-logs")
            .help("Enable OpenTelemetry logging")
            .global(true)
            .hide(true)
            .action(clap::ArgAction::SetTrue),
        Arg::new("otel-exporter")
            .long("otel-exporter")
            .help("The OpenTelemetry exporter to use (stdout, otlp)")
            .num_args(1)
            .global(true)
            .hide(true)
            .default_value("stdout")
            .env("OTEL_TRACES_EXPORTER")
            .value_parser(clap::builder::PossibleValuesParser::new(&[
                "stdout", "otlp",
            ])),
        Arg::new("otel-exporter-endpoint")
            .long("otel-exporter-endpoint")
            .help("The endpoint to use for the OTLP exporter (required if --otel-exporter=otlp)")
            .num_args(1)
            .global(true)
            .hide(true)
            .requires_if("otlp", "otel-exporter")
            .env("OTEL_EXPORTER_OTLP_ENDPOINT")
            .value_parser(clap::builder::NonEmptyStringValueParser::new()),
        Arg::new("otel-exporter-protocol")
            .long("otel-exporter-protocol")
            .help("The protocol to use for the OTLP exporter (http/protobuf, grpc)")
            .num_args(1)
            .global(true)
            .hide(true)
            .default_value("http")
            .requires_if("otlp", "otel-exporter")
            .env("OTEL_EXPORTER_OTLP_PROTOCOL")
            .value_parser(clap::builder::PossibleValuesParser::new(&[
                "http",
                "http/protobuf",
                "grpc",
            ])),
    ]
}
