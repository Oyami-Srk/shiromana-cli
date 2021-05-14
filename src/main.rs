#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use std::path::{Path, PathBuf};

use confy;
use console;
use console::style;
use directories_next;
use serde::{Deserialize, Serialize};
use shiromana_rs::library::{Library, LibrarySummary};

use add_image::*;
use command::*;
use library::*;
use prompter::*;
use std::error::Error;

mod add_image;
mod command;
mod library;
mod prompter;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    version: u8,
    library_path: String,
    library_name: String,
}

impl ::std::default::Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            library_path: directories_next::UserDirs::new()
                .unwrap()
                .picture_dir()
                .unwrap_or(Path::new(""))
                .to_str()
                .unwrap_or("")
                .to_string(),
            library_name: "shiro-lib".to_string(),
        }
    }
}

fn load_config(config_path: Option<PathBuf>) -> Result<(AppConfig, Library), Box<dyn Error>> {
    let config_path = config_path.unwrap_or(
        match confy::get_configuration_file_path("shiromana-cli", "config") {
            Ok(v) => v,
            Err(e) => {
                panic!("Cannot get the path to configuration file due to {}", e);
            }
        },
    );

    #[cfg(feature = "purge-every-time")]
    #[cfg(debug_assertions)]
    {
        println!(
            "{}",
            style(
                "Debug environment will remove configuration file and default Library every time."
            )
            .blue()
        );
        std::fs::remove_dir_all(config_path.clone().parent().unwrap()).unwrap_or(());
        let default = AppConfig::default();
        std::fs::remove_dir_all(default.library_path + "/" + &default.library_name + ".mlib")
            .unwrap_or(());
    }

    let (config, library) = if config_path.exists() {
        let config = match confy::load("shiromana-cli", "config") {
            Ok(v) => v,
            Err(e) => {
                panic!(
                    "Cannot load configuration file at {} due to {}.",
                    config_path.to_str().unwrap(),
                    e
                );
            }
        };
        let library = match open_library(&config) {
            Ok(v) => v,
            Err(e) => panic!(
                "Error when opening Library {} at {} due to {}.",
                config.library_name + ".mlib",
                config.library_path,
                e
            ),
        };
        (config, library)
    } else {
        let config = AppConfig {
            version: 1,
            library_path: match ask_for_location(true, AppConfig::default().library_path) {
                Ok(v) => v,
                Err(e) => {
                    panic!("User input processing error due to: {}", e);
                }
            },
            library_name: match ask_for_library_name(AppConfig::default().library_name) {
                Ok(v) => v,
                Err(e) => {
                    panic!("User input processing error due to: {}", e);
                }
            },
        };
        confy::store("shiromana-cli", "config", &config).unwrap();
        let library = match create_library(&config) {
            Ok(v) => v,
            Err(e) => panic!(
                "Error when creating Library {} at {} due to {}.",
                config.library_name + ".mlib",
                config.library_path,
                e
            ),
        };
        (config, library)
    };
    Ok((config, library))
}

use clap::Clap;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Shiroko <hhx.xxm@gmail.com>")]
struct Opts {
    #[clap(short, long, validator(is_existed_as_file))]
    config: Option<String>,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap()]
    Info(Info),
    Add(Add),
}

#[derive(Clap)]
pub struct Info {
    media: Option<String>,
    #[clap(short, long)]
    detail: bool,
}

#[derive(Clap)]
pub struct Add {
    #[clap(short, long)]
    _move: bool,
    #[clap(short, long)]
    comment: Option<String>,
    #[clap(short, long)]
    title: Option<String>,
    #[clap(validator(is_existed_as_file))]
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    let config_path = opts.config.map(|v| PathBuf::from(v));
    let (cfg, lib) = load_config(config_path)?;
    match opts.subcmd {
        SubCommand::Info(opt) => do_info(opt, cfg, lib),
        SubCommand::Add(opt) => do_add(opt, cfg, lib),
    }
}

fn is_existed_as_file(v: &str) -> Result<(), String> {
    let p = Path::new(v);
    if p.exists() && p.is_file() {
        Ok(())
    } else {
        if p.exists() {
            Err("Not a file.".to_string())
        } else {
            Err("Not Exists".to_string())
        }
    }
}
