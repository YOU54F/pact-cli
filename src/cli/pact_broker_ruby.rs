use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{Arg, ArgMatches, Command, Error};
use std::{
    env, fs,
    io::{Read, Write},
    path::Path,
    process::{Command as Cmd, ExitStatus},
};

pub fn add_ruby_broker_subcommand() -> Command {
    Command::new("ruby")
        .about("Install & Run the Pact Broker using system Ruby in $HOME/.pact/pact-broker")
        .subcommand(
            Command::new("start")
                .about("Setup and Start the Pact Broker")
                .arg(
                    Arg::new("detach")
                        .short('d')
                        .long("detach")
                        .num_args(0)
                        .action(clap::ArgAction::SetTrue)
                        .help("Run the Pact Broker in the background"),
                ),
        )
        .subcommand(Command::new("stop").about("Stop the Pact Broker"))
        .subcommand(Command::new("remove").about("Remove the Pact Broker"))
        .subcommand(Command::new("info").about("Info about the Pact Broker"))
}

fn check_ruby_version() -> Result<(), String> {
    let output = Cmd::new("ruby")
        .arg("-e")
        .arg("print RUBY_VERSION")
        .output()
        .map_err(|_| "Ruby is not installed or not in PATH.".to_string())?;

    let version_str = String::from_utf8_lossy(&output.stdout);
    let version_parts: Vec<&str> = version_str.split('.').collect();
    if version_parts.len() < 2 {
        return Err("Could not determine Ruby version.".to_string());
    }
    let major = version_parts[0].parse::<u32>().unwrap_or(0);
    let minor = version_parts[1].parse::<u32>().unwrap_or(0);

    if major > 3 || (major == 3 && minor >= 1) {
        Ok(())
    } else {
        Err(format!(
            "Ruby version 3.1 or greater is required. Found version {}.",
            version_str
        ))
    }
}

fn check_bundler_installed() -> Result<(), String> {
    // Use 'ruby -S bundle' for better cross-platform compatibility
    let output = Cmd::new("ruby")
        .arg("-S")
        .arg("bundle")
        .arg("--version")
        .output()
        .map_err(|_| "Bundler is not installed or not in PATH.".to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err("Bundler is not installed or not in PATH.".to_string())
    }
}

fn write_gemfile_and_config(broker_dir: &Path) -> std::io::Result<()> {
    let gemfile_content = r#"source 'https://rubygems.org'

gem 'rake'
gem 'pact_broker'
if Gem.win_platform?
  gem 'sqlite3', force_ruby_platform: true
else
  gem 'sqlite3'
end
gem 'puma'
gem "padrino-core", ">= 0.16.0.pre3" # Required for the pact_broker UI.
gem "pact-support"
# required for ruby 3.4 (removed from std gems)
gem "mutex_m"
gem "csv"
"#;

    let config_ru_content = r#"require 'logger'
require 'sequel'
require 'pact_broker'

DATABASE_CREDENTIALS = {adapter: "sqlite", database: "pact_broker_database.sqlite3", :encoding => 'utf8'}

#  run via one of the following:
#  
#  $ bundle exec rackup -s thin
#  $ bundle exec rackup -s puma
#  $ bundle exec rackup -s webrick
#  
#  Note: if using thin, publishing results will fail with the rust verifier, as it requires the Accept-Charset header
#  to be set to utf-8. Use puma or webrick instead, until change proposed/merged in pact-rust

app = PactBroker::App.new do | config |
  config.log_stream = "stdout"
  # config.base_urls = "http://localhost:9292 http://127.0.0.1:9292 http://0.0.0.0:9292"
  # config.database_url = "sqlite:////tmp/pact_broker_database.sqlite3"
  config.database_connection = Sequel.connect(DATABASE_CREDENTIALS.merge(:logger => config.logger))
end

run app
"#;

    fs::create_dir_all(broker_dir)?;
    fs::write(broker_dir.join("Gemfile"), gemfile_content)?;
    fs::write(broker_dir.join("config.ru"), config_ru_content)?;
    Ok(())
}

pub fn run(args: &ArgMatches) -> Result<(), String> {
    let home_dir = home::home_dir().ok_or("Could not determine home directory.")?;
    let broker_dir = home_dir.join(".pact/pact-broker");
    let pid_file_path = broker_dir.join("broker.pid");

    match args.subcommand() {
        Some(("start", args)) => {
            check_ruby_version()?;
            check_bundler_installed()?;
            write_gemfile_and_config(&broker_dir)
                .map_err(|e| format!("Failed to write Gemfile/config.ru: {}", e))?;

            println!("🚀 Running bundle install in {}", broker_dir.display());
            let status = Cmd::new("ruby")
                .arg("-S")
                .arg("bundle")
                .arg("install")
                .current_dir(&broker_dir)
                .status()
                .map_err(|_| "Failed to run bundle install".to_string())?;
            if !status.success() {
                return Err(
                    "⚠️  bundle install failed. Please check your Ruby and Bundler setup."
                        .to_string(),
                );
            }

            println!("🚀 Starting Pact Broker with Puma...");
            let mut child_cmd = Cmd::new("ruby");
            child_cmd.arg("-S").arg("bundle");
            child_cmd
                .arg("exec")
                .arg("puma")
                .arg("--pidfile")
                .arg(&pid_file_path)
                .current_dir(&broker_dir);

            let mut child = child_cmd
                .spawn()
                .map_err(|_| "Failed to start Pact Broker".to_string())?;
            let pid = child.id();
            println!("🚀 Pact Broker is running on http://localhost:9292");
            println!("🚀 PID: {}", pid);
            println!("🚀 PID file: {}", pid_file_path.display());
            let mut pid_file_contents = String::from("unknown");
            while !pid_file_contents.chars().all(char::is_numeric) {
                std::thread::sleep(std::time::Duration::from_secs(1));
                pid_file_contents =
                    fs::read_to_string(&pid_file_path).unwrap_or_else(|_| String::from("unknown"));
            }
            println!("Traveling Broker PID: {}", pid_file_contents);

            let detach = args.get_flag("detach");
            if detach {
                println!("🚀 Running in the background");
                return Ok(());
            } else {
                while child.try_wait().unwrap().is_none() {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                let _ = child.kill();
                let pid_file = fs::File::open(&pid_file_path);
                match pid_file {
                    Ok(mut file) => {
                        let mut pid = String::new();
                        file.read_to_string(&mut pid).unwrap();
                        let pid = pid.trim().parse::<u32>().unwrap();
                        println!("🚀 Stopping Pact Broker with PID: {}", pid);
                        #[cfg(windows)]
                        Cmd::new("taskkill")
                            .arg("/F")
                            .arg("/PID")
                            .arg(pid.to_string())
                            .output()
                            .expect("Failed to stop the process");
                    }
                    Err(_) => {
                        println!("PID file not found");
                    }
                }
                let _ = fs::remove_file(&pid_file_path);
                return Ok(());
            }
        }
        Some(("stop", _args)) => {
            let mut file = fs::File::open(&pid_file_path)
                .map_err(|_| "⚠️ Pact Broker is not running".to_string())?;
            let mut pid = String::new();
            file.read_to_string(&mut pid).unwrap();
            let pid = pid.trim().parse::<u32>().unwrap();
            println!("🚀 Stopping Pact Broker with PID: {}", pid);
            #[cfg(windows)]
            Cmd::new("taskkill")
                .arg("/F")
                .arg("/PID")
                .arg(pid.to_string())
                .output()
                .expect("⚠️ Failed to stop the broker");

            #[cfg(not(windows))]
            Cmd::new("kill")
                .arg(pid.to_string())
                .output()
                .expect("⚠️ Failed to stop the broker");
            let _ = fs::remove_file(&pid_file_path);
            println!("🛑 Pact Broker stopped");
            Ok(())
        }
        Some(("remove", _args)) => {
            let matches = add_ruby_broker_subcommand().get_matches_from(["ruby", "stop"]);
            let _ = run(&matches);
            if let Ok(metadata) = fs::metadata(&broker_dir) {
                if metadata.is_dir() {
                    if let Err(err) = fs::remove_dir_all(&broker_dir) {
                        println!("Failed to remove broker_dir: {}", err);
                    } else {
                        println!("broker_dir removed successfully");
                    }
                }
            } else {
                println!("broker_dir {} not found", broker_dir.display());
            }
            Ok(())
        }
        Some(("info", _args)) => {
            fn check_directory_exists(directory: &Path) -> bool {
                directory.exists()
            }

            let pact_broker_ruby_exists = check_directory_exists(&broker_dir);

            println!("Pact broker directory exists: {}", pact_broker_ruby_exists);

            fn get_ruby_version() -> std::io::Result<String> {
                let output = Cmd::new("ruby").arg("-v").output()?;
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            }

            println!("Ruby version: {:?}", get_ruby_version());

            fn check_pid_file_exists(pid_file_path: &Path) -> bool {
                pid_file_path.exists()
            }

            let pact_broker_pid_file_exists = check_pid_file_exists(&pid_file_path);
            println!("Pact broker pid exists: {}", pact_broker_pid_file_exists);

            fn get_pid_from_file(pid_file_path: &Path) -> Option<u32> {
                if let Ok(mut file) = fs::File::open(pid_file_path) {
                    let mut pid = String::new();
                    file.read_to_string(&mut pid).unwrap();
                    Some(pid.trim().parse::<u32>().unwrap())
                } else {
                    None
                }
            }

            let pact_broker_pid_exists = get_pid_from_file(&pid_file_path);
            println!("Pact broker pid: {:?}", pact_broker_pid_exists);
            Ok(())
        }
        _ => {
            println!("⚠️  No option provided, try running ruby --help");
            Ok(())
        }
    }
}
