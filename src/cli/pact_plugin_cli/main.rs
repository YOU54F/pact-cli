use std::path::PathBuf;
use std::process::ExitCode;
use std::{env, fs};
// use std::str::FromStr;

use anyhow::anyhow;
use clap::{Arg, ArgMatches, Command, Subcommand};
// use clap::{ArgMatches, Args, FromArgMatches, Parser, Subcommand};
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use itertools::Itertools;
use pact_plugin_driver::plugin_models::PactPluginManifest;
use requestty::OnEsc;
use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;

// use crate::cli;

// use super::main::install;
mod install;
mod list;
mod repository;
use list::{plugin_list};



pub fn add_plugin_cli_subcommand() -> Command {
    Command::new("plugin") 
    .arg_required_else_help(true)
    .about("CLI utility for Pact plugins")
    .arg(Arg::new("yes")
        .short('y')
        .long("yes")
        .num_args(0)
        .action(clap::ArgAction::SetTrue)
        .help("Automatically answer Yes for all prompts"))
    .arg(Arg::new("debug")
        .short('d')
        .long("debug")
        .num_args(0)
        .action(clap::ArgAction::SetTrue)
        .help("Enable debug level logs"))
    .arg(Arg::new("trace")
        .short('t')
        .long("trace")
        .num_args(0)
        .action(clap::ArgAction::SetTrue)
        .help("Enable trace level logs"))
    .arg(Arg::new("cli_version")
        .short('v')
        .long("version")
        .help("Print CLI version")
        .num_args(0))
    .subcommand(Command::new("list")
        .about("List the installed plugins")
        .arg_required_else_help(true)
        .subcommand(Command::new("installed")
            .about("List installed plugins"))
        .subcommand(Command::new("known")
            .about("List known plugins")
            .arg(Arg::new("show_all_versions")
                .short('a')
                .long("show-all-versions")
                .help("Display all versions of the known plugins")
                .action(clap::ArgAction::SetTrue)
            )
            ))
    .subcommand(Command::new("env")
        .about("Print out the Pact plugin environment config"))
    .subcommand(Command::new("install")
        .about("Install a plugin \n\nA plugin can be either installed from a URL, or for a known plugin, by name (and optionally version)")
        .arg_required_else_help(true)
        .arg(Arg::new("source_type")
            .short('t')
            .long("source-type")
            .num_args(1)
            .value_name("SOURCE_TYPE")
            .help("The type of source to fetch the plugin files from. Will default to Github releases.")
            .value_parser(clap::builder::PossibleValuesParser::new(&["github"])))
        .arg(Arg::new("yes")
            .short('y')
            .long("yes")
            .action(clap::ArgAction::SetTrue)
            .help("Automatically answer Yes for all prompts"))
        .arg(Arg::new("skip_if_installed")
            .long("skip-if-installed")
            .action(clap::ArgAction::SetTrue)
            .short('s')
            .help("Skip installing the plugin if the same version is already installed"))
        .arg(Arg::new("source")
            .help("Where to fetch the plugin files from. This should be a URL or the name of a known plugin.")
            .value_name("SOURCE")
            .required(true))
        .arg(Arg::new("version")
            .long("version")
            .short('v')
            .num_args(1)
            .help("The version to install. This is only used for known plugins.")
            .value_name("VERSION")))  
    .subcommand(Command::new("remove")
        .about("Remove a plugin")
        .arg(Arg::new("yes")
            .short('y')
            .long("yes")
            .action(clap::ArgAction::SetTrue)
            .help("Automatically answer Yes for all prompts"))
        .arg(Arg::new("name")
            .value_name("NAME")
            .required(true)
            .help("Plugin name"))
        .arg(Arg::new("version")
            .value_name("VERSION")
            // .value_parser(clap::builder::NonEmptyStringValueParser::new())
            .help("Plugin version. Not required if there is only one plugin version."))

    )
    .subcommand(Command::new("enable")
    .arg_required_else_help(true)
        .about("Enable a plugin version")
        .arg(Arg::new("name")
        .required(true)
        .help("Plugin name"))
    .arg(Arg::new("version")
        .help("Plugin version. Not required if there is only one plugin version.")
        .value_name("VERSION"))
    )
    .subcommand(Command::new("disable")
    .arg_required_else_help(true)
        .about("Disable a plugin version")
        .arg(Arg::new("name")
        .required(true)
        .help("Plugin name"))
    .arg(Arg::new("version")
        .help("Plugin version. Not required if there is only one plugin version.")
        .value_name("VERSION"))
    )
    .subcommand(Command::new("repository")
        .arg_required_else_help(true)
        .about("Sub-commands for dealing with a plugin repository")
        .subcommand(Command::new("validate")
        .about("Check the consistency of the repository index file")
        .arg(Arg::new("filename")
        .value_name("FILENAME")
        .required(true)
            .help("Filename to validate")))
    .subcommand(Command::new("new")
        .about("Create a new blank repository index file")
        .arg(Arg::new("filename")
        .value_name("FILENAME")
            .help("Filename to use for the new file. By default will use repository.index"))
        .arg(Arg::new("overwrite")
            .short('o')
            .long("overwrite")
            .num_args(0)
            .help(" Overwrite any existing file?"))
        )
        .subcommand(Command::new("add-plugin-version")
            .about("Add a plugin version to the index file (will update existing entry)")
            .arg_required_else_help(true)
            .subcommand_required(true)
        .subcommand(Command::new("file")
            .about("Add an entry for a local plugin manifest file to the repository file")
            .arg(Arg::new("repository_file")
            .value_name("REPOSITORY_FILE")
                .required(true)
                .help("Repository index file to update"))
            .arg(Arg::new("name")
            .value_name("FILE")
            .required(true)
                .help("Path to the local plugin manifest file")))
        .subcommand(Command::new("git-hub")
            .about("Add an entry for a GitHub Release to the repository file")
            .arg(Arg::new("repository_file")
            .value_name("REPOSITORY_FILE")
                .required(true)
                .help("Repository index file to update"))
            .arg(Arg::new("url")
            .value_name("URL")
            .required(true)
                .help("Base URL for GitHub APIs, will default to https://api.github.com/repos/")))
        )
        .subcommand(Command::new("add-all-plugin-versions")
        .about("Add all versions of a plugin to the index file (will update existing entries)")
        .arg(Arg::new("repository_file")
            .value_name("REPOSITORY_FILE")
            .required(true)
            .help("Repository index file to update"))
        .arg(Arg::new("owner")
            .value_name("OWNER")
            .required(true)
            .help("Repository owner to load versions from"))
        .arg(Arg::new("repository")
            .value_name("REPOSITORY")
            .required(true)
            .help("Repository to load versions from"))
        .arg(Arg::new("base_url")
            .value_name("BASE_URL")
            .help("Base URL for GitHub APIs, will default to https://api.github.com/repos/")))
    .subcommand(Command::new("yank-version")
        .about("Remove a plugin version from the index file"))
    .subcommand(Command::new("list")
        .about("List all plugins found in the index file")
        .arg(Arg::new("filename")
        .value_name("FILENAME")
            .required(true)
            .help("Filename to list entries from")))
    .subcommand(Command::new("list-versions")
        .about("List all plugin versions found in the index file")
        .arg(Arg::new("filename")
        .value_name("FILENAME")
            .required(true)
            .help("Filename to list versions from"))
            .arg(Arg::new("name")
            .value_name("NAME")
            .required(true)
            .help("Plugin entry to list versions for"))   ) 

    )
}

// #[derive(Parser, Debug)]
// #[clap(about, version)]
// #[command(disable_version_flag(true))]
// struct Cli {
//   #[clap(short, long)]
//   /// Automatically answer Yes for all prompts
//   yes: bool,

//   #[clap(short, long)]
//   /// Enable debug level logs
//   debug: bool,

//   #[clap(short, long)]
//   /// Enable trace level logs
//   trace: bool,

//   #[clap(subcommand)]
//   command: Commands,

//   #[clap(short = 'v', long = "version", action = clap::ArgAction::Version)]
//   /// Print CLI version
//   cli_version: Option<bool>
// }

// #[derive(Subcommand, Debug)]
// enum Commands {
//   /// List installed or available plugins
//   #[command(subcommand)]
//   List(ListCommands),

//   /// Print out the Pact plugin environment config
//   Env,

//   /// Install a plugin
//   ///
//   /// A plugin can be either installed from a URL, or for a known plugin, by name (and optionally
//   /// version).
//   Install {
//     /// The type of source to fetch the plugin files from. Will default to Github releases.
//     ///
//     /// Valid values: github
//     #[clap(short = 't', long)]
//     source_type: Option<InstallationSource>,

//     #[clap(short, long)]
//     /// Automatically answer Yes for all prompts
//     yes: bool,

//     #[clap(short, long)]
//     /// Skip installing the plugin if the same version is already installed
//     skip_if_installed: bool,

//     /// Where to fetch the plugin files from. This should be a URL or the name of a known plugin.
//     source: String,

//     #[clap(short, long)]
//     /// The version to install. This is only used for known plugins.
//     version: Option<String>
//   },

//   /// Remove a plugin
//   Remove {
//     #[clap(short, long)]
//     /// Automatically answer Yes for all prompts
//     yes: bool,

//     /// Plugin name
//     name: String,

//     /// Plugin version. Not required if there is only one plugin version.
//     version: Option<String>
//   },

//   /// Enable a plugin version
//   Enable {
//     /// Plugin name
//     name: String,

//     /// Plugin version. Not required if there is only one plugin version.
//     version: Option<String>
//   },

//   /// Disable a plugin version
//   Disable {
//     /// Plugin name
//     name: String,

//     /// Plugin version. Not required if there is only one plugin version.
//     version: Option<String>
//   },

//   /// Sub-commands for dealing with a plugin repository
//   #[command(subcommand)]
//   Repository(RepositoryCommands)
// }

// #[derive(Subcommand, Debug)]
// pub enum ListCommands {
//   /// List installed plugins
//   Installed,

//   /// List known plugins
//   Known {
//     /// Display all versions of the known plugins
//     #[clap(short, long)]
//     show_all_versions: bool
//   }
// }

// #[derive(Subcommand, Debug)]
// enum RepositoryCommands {
//   /// Check the consistency of the repository index file
//   Validate {
//     /// Filename to validate
//     filename: String
//   },

//   /// Create a new blank repository index file
//   New {
//     /// Filename to use for the new file. By default will use repository.index
//     filename: Option<String>,

//     #[clap(short, long)]
//     /// Overwrite any existing file?
//     overwrite: bool
//   },

//   /// Add a plugin version to the index file (will update existing entry)
//   #[command(subcommand)]
//   AddPluginVersion(PluginVersionCommand),

//   /// Add all versions of a plugin to the index file (will update existing entries)
//   AddAllPluginVersions {
//     /// Repository index file to update
//     repository_file: String,

//     /// Repository owner to load versions from
//     owner: String,

//     /// Repository to load versions from
//     repository: String,

//     /// Base URL for GitHub APIs, will default to https://api.github.com/repos/
//     base_url: Option<String>
//   },

//   /// Remove a plugin version from the index file
//   YankVersion,

//   /// List all plugins found in the index file
//   List {
//     /// Filename to list entries from
//     filename: String
//   },

//   /// List all plugin versions found in the index file
//   ListVersions{
//     /// Filename to list versions from
//     filename: String,

//     /// Plugin entry to list versions for
//     name: String
//   }
// }

#[derive(Subcommand, Debug)]
enum PluginVersionCommand {
    /// Add an entry for a local plugin manifest file to the repository file
    File {
        repository_file: String,
        file: String,
    },

    /// Add an entry for a GitHub Release to the repository file
    GitHub {
        repository_file: String,
        url: String,
    },
}

/// Installation source to fetch plugins files from
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum InstallationSource {
    /// Install the plugin from a Github release page.
    Github,
}

// impl FromStr for InstallationSource {
//   type Err = anyhow::Error;
//   fn from_str(s: &str) -> Result<Self, Self::Err> {
//     if s.to_lowercase() == "github" {
//       Ok(InstallationSource::Github)
//     } else {
//       Err(anyhow!("'{}' is not a valid installation source", s))
//     }
//   }
// }

// fn main() -> Result<(), ExitCode> {
//   let cli = Cli::parse();

//   let log_level = if cli.trace {
//     Level::TRACE
//   } else if cli.debug {
//     Level::DEBUG
//   } else {
//     Level::WARN
//   };
//   let subscriber = FmtSubscriber::builder()
//     .with_max_level(log_level)
//     .finish();

//   if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
//     eprintln!("WARN: Failed to initialise global tracing subscriber - {err}");
//   };

//   let result = match &cli.command {
//     Commands::List(command) => list_plugins(command),
//     Commands::Env => print_env(),
//     Commands::Install { yes, skip_if_installed, source, source_type, version } => {
//       install::install_plugin(source, source_type, *yes || cli.yes, *skip_if_installed, version)
//     },
//     Commands::Remove { yes, name, version } => remove_plugin(name, version, *yes || cli.yes),
//     Commands::Enable { name, version } => enable_plugin(name, version),
//     Commands::Disable { name, version } => disable_plugin(name, version),
//     Commands::Repository(command) => repository::handle_command(command)
//   };

//   result.map_err(|err| {
//     error!("error - {}", err);
//     ExitCode::FAILURE
//   })
// }

pub fn run(args: &ArgMatches) -> Result<(), ExitCode> {
    let log_level: Level = if args.get_flag("trace") == true {
        Level::TRACE
    } else if args.get_flag("debug") == true {
        Level::DEBUG
    } else {
        Level::WARN
    };
    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();

    if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("WARN: Failed to initialise global tracing subscriber - {err}");
    };

    let result = match args.subcommand() {
        Some(("list", args)) => list::list_plugins(args),
        Some(("env", _)) => print_env(),
        Some(("install", args)) => install::install_plugin(
            args.get_one::<String>("source").unwrap(),
            &args
                .get_one::<InstallationSource>("source_type")
                .map(|st| st.clone()),
            args.get_flag("yes"),
            args.get_flag("skip_if_installed"),
            args.get_one::<Option<String>>("version").unwrap_or(&None),
        ),
        Some(("remove", args)) => remove_plugin(
            args.get_one::<String>("name").unwrap(),
            &args
                .get_one::<String>("version")
                .map(|version| version.to_string()),
            args.get_flag("yes"),
        ),
        Some(("enable", args)) => enable_plugin(
            args.get_one::<String>("name").unwrap(),
            &args
                .get_one::<String>("version")
                .map(|version| version.to_string()),
        ),
        Some(("disable", args)) => disable_plugin(
            args.get_one::<String>("name").unwrap(),
            &args
                .get_one::<String>("version")
                .map(|version| version.to_string()),
        ),
        Some(("repository", args)) => repository::handle_command(args),
        None => unimplemented!("Handle empty ArgMatches case"),
        Some((&_, _)) => unimplemented!("Handle unknown subcommand case"), // Commands::List(command) => list_plugins(command),
                                                                           // Commands::Env => print_env(),
                                                                           // Commands::Install { yes, skip_if_installed, source, source_type, version } => {
                                                                           //   install::install_plugin(source, source_type, *yes || cli.yes, *skip_if_installed, version)
                                                                           // },
                                                                           // Commands::Remove { yes, name, version } => remove_plugin(name, version, *yes || cli.yes),
                                                                           // Commands::Enable { name, version } => enable_plugin(name, version),
                                                                           // Commands::Disable { name, version } => disable_plugin(name, version),
                                                                           // Commands::Repository(command) => repository::handle_command(command)
    };

    result.map_err(|err| {
        error!("error - {}", err);
        ExitCode::FAILURE
    })
}

fn remove_plugin(
    name: &String,
    version: &Option<String>,
    override_prompt: bool,
) -> anyhow::Result<()> {
    let matches = find_plugin(name, version)?;
    if matches.len() == 1 {
        if let Some((manifest, _, _)) = matches.first() {
            if override_prompt || prompt_delete(manifest) {
                fs::remove_dir_all(manifest.plugin_dir.clone())?;
                println!(
                    "Removed plugin with name '{}' and version '{}'",
                    manifest.name, manifest.version
                );
            } else {
                println!("Aborting deletion of plugin.");
            }
            Ok(())
        } else {
            Err(anyhow!(
                "Internal error, matches.len() == 1 but first() == None"
            ))
        }
    } else if matches.len() > 1 {
        Err(anyhow!(
            "There is more than one plugin version for '{}', please also provide the version",
            name
        ))
    } else if let Some(version) = version {
        Err(anyhow!(
            "Did not find a plugin with name '{}' and version '{}'",
            name,
            version
        ))
    } else {
        Err(anyhow!("Did not find a plugin with name '{}'", name))
    }
}

fn prompt_delete(manifest: &PactPluginManifest) -> bool {
    let question = requestty::Question::confirm("delete_plugin")
        .message(format!(
            "Are you sure you want to delete plugin with name '{}' and version '{}'?",
            manifest.name, manifest.version
        ))
        .default(false)
        .on_esc(OnEsc::Terminate)
        .build();
    if let Ok(result) = requestty::prompt_one(question) {
        if let Some(result) = result.as_bool() {
            result
        } else {
            false
        }
    } else {
        false
    }
}

fn disable_plugin(name: &String, version: &Option<String>) -> anyhow::Result<()> {
    let matches = find_plugin(name, version)?;
    if matches.len() == 1 {
        if let Some((manifest, file, status)) = matches.first() {
            if !*status {
                println!(
                    "Plugin '{}' with version '{}' is already disabled.",
                    manifest.name, manifest.version
                );
            } else {
                fs::rename(file, file.with_file_name("pact-plugin.json.disabled"))?;
                println!(
                    "Plugin '{}' with version '{}' is now disabled.",
                    manifest.name, manifest.version
                );
            }
            Ok(())
        } else {
            Err(anyhow!(
                "Internal error, matches.len() == 1 but first() == None"
            ))
        }
    } else if matches.len() > 1 {
        Err(anyhow!(
            "There is more than one plugin version for '{}', please also provide the version",
            name
        ))
    } else if let Some(version) = version {
        Err(anyhow!(
            "Did not find a plugin with name '{}' and version '{}'",
            name,
            version
        ))
    } else {
        Err(anyhow!("Did not find a plugin with name '{}'", name))
    }
}

fn find_plugin(
    name: &String,
    version: &Option<String>,
) -> anyhow::Result<Vec<(PactPluginManifest, PathBuf, bool)>> {
    let vec = plugin_list()?;
    Ok(vec
        .iter()
        .filter(|(manifest, _, _)| {
            if let Some(version) = version {
                manifest.name == *name && manifest.version == *version
            } else {
                manifest.name == *name
            }
        })
        .map(|(m, p, s)| (m.clone(), p.clone(), *s))
        .collect_vec())
}

fn enable_plugin(name: &String, version: &Option<String>) -> anyhow::Result<()> {
    let matches = find_plugin(name, version)?;
    if matches.len() == 1 {
        if let Some((manifest, file, status)) = matches.first() {
            if *status {
                println!(
                    "Plugin '{}' with version '{}' is already enabled.",
                    manifest.name, manifest.version
                );
            } else {
                fs::rename(file, file.with_file_name("pact-plugin.json"))?;
                println!(
                    "Plugin '{}' with version '{}' is now enabled.",
                    manifest.name, manifest.version
                );
            }
            Ok(())
        } else {
            Err(anyhow!(
                "Internal error, matches.len() == 1 but first() == None"
            ))
        }
    } else if matches.len() > 1 {
        Err(anyhow!(
            "There is more than one plugin version for '{}', please also provide the version",
            name
        ))
    } else if let Some(version) = version {
        Err(anyhow!(
            "Did not find a plugin with name '{}' and version '{}'",
            name,
            version
        ))
    } else {
        Err(anyhow!("Did not find a plugin with name '{}'", name))
    }
}

fn print_env() -> anyhow::Result<()> {
    let mut table = Table::new();

    let (plugin_src, plugin_dir) = resolve_plugin_dir();

    table
        .load_preset(UTF8_FULL)
        .set_header(vec!["Configuration", "Source", "Value"])
        .add_row(vec![
            "Plugin Directory",
            plugin_src.as_str(),
            plugin_dir.as_str(),
        ]);

    println!("{table}");

    Ok(())
}

fn resolve_plugin_dir() -> (String, String) {
    let home_dir = home::home_dir()
        .map(|dir| dir.join(".pact/plugins"))
        .unwrap_or_default();
    match env::var_os("PACT_PLUGIN_DIR") {
        None => (
            "$HOME/.pact/plugins".to_string(),
            home_dir.display().to_string(),
        ),
        Some(dir) => {
            let plugin_dir = dir.to_string_lossy();
            if plugin_dir.is_empty() {
                (
                    "$HOME/.pact/plugins".to_string(),
                    home_dir.display().to_string(),
                )
            } else {
                ("$PACT_PLUGIN_DIR".to_string(), plugin_dir.to_string())
            }
        }
    }
}

// #[cfg(test)]
// mod tests;
