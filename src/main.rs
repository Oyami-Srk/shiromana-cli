#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

#[macro_use]
extern crate lazy_static;

use std::path::{Path, PathBuf};

use confy;
use console;
use console::style;
use directories_next;
use serde::{Deserialize, Serialize};
use shiromana_rs::library::{Library, LibrarySummary};

use add_image::*;
use command::*;
use ctrlc;
use library::*;
use prompter::*;
use std::error::Error;
use std::str::FromStr;
use std::sync::mpsc::channel;

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

#[cfg(debug_assertions)]
fn purge_library(config: &AppConfig) {
    std::fs::remove_dir_all(
        config.library_path.clone() + "/" + config.library_name.as_str() + ".mlib",
    )
    .unwrap_or(());
}

#[cfg(debug_assertions)]
fn purge(config_path: &PathBuf) {
    println!(
        "{}",
        style("Debug environment will remove configuration file and default Library every time.")
            .blue()
    );
    std::fs::remove_dir_all(config_path.clone().parent().unwrap()).unwrap_or(());
    purge_library(&AppConfig::default());
}

#[cfg(debug_assertions)]
fn recreate(config: &AppConfig) {
    println!(
        "{}",
        style("Creating configuration file and default library.").blue()
    );
    confy::store("shiromana-cli", "config", &config).unwrap();
    let _library = match create_library(&config) {
        Ok(v) => v,
        Err(e) => panic!(
            "Error when creating Library {} at {} due to {}.",
            config.library_name.clone() + ".mlib",
            config.library_path,
            e
        ),
    };
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
    purge(&config_path);
    #[cfg(feature = "auto-create")]
    recreate(&AppConfig::default());

    let (config, library) = if config_path.exists() {
        let config = match confy::load_path(&config_path) {
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
        let config_dir = config_path.parent();
        if config_dir.is_none() {
            panic!("Config file's directory cannot be resolved. Why?");
        } else {
            if !config_dir.unwrap().exists() {
                std::fs::create_dir_all(config_dir.unwrap())?;
            }
        }
        confy::store_path(config_path, &config).unwrap();
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

use clap::{App, ArgGroup, Clap, ValueHint};
use shiromana_rs::media::{Media, MediaType};
use shiromana_rs::misc::Uuid;
use std::process::exit;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Shiroko <hhx.xxm@gmail.com>")]
struct Opts {
    #[clap(short, long, value_hint = ValueHint::FilePath)]
    config: Option<String>,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap()]
    Info(Info),
    Add(Add),
    Create(Create),
    Clean,
}

#[derive(Clap)]
pub struct Info {
    media: Option<String>,
    #[clap(short, long)]
    detail: bool,
}

#[derive(Clap, Debug)]
#[clap(group = ArgGroup::new("input").required(true), group = ArgGroup::new("series_g").required(false))]
pub struct Add {
    #[clap(short, long)]
    _move: bool,
    #[clap(short, long)]
    comment: Option<String>,
    #[clap(short, long)]
    title: Option<String>,
    #[clap(short = 'k', long, validator(is_valid_media_type))]
    _type: Option<MediaType>,
    #[clap(name = "FILE", parse(from_os_str), value_hint = ValueHint::FilePath, validator(is_existed_as_file), group = "input")]
    file: Vec<PathBuf>,
    #[clap(short, long, name = "INPUT", parse(from_os_str), value_hint = ValueHint::FilePath, validator(is_existed_as_file), group = "input")]
    input: Option<PathBuf>,
    #[clap(short, long, group = "series_g")]
    series: Option<Uuid>,
    #[clap(short, long, group = "series_g", name = "series name")]
    new_series: Option<String>,
    #[clap(long)]
    sorted: bool,
}

pub enum CreateType {
    Series,
    Tag,
}

#[derive(Clap)]
pub struct Create {
    _type: CreateType,
    title: String,
    #[clap(short, long)]
    comment: Option<String>,
    #[clap(short, long)]
    uuid_only: bool,
}

impl FromStr for CreateType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "series" => Ok(Self::Series),
            "tag" => Ok(Self::Tag),
            "s" => Ok(Self::Series),
            "t" => Ok(Self::Tag),
            _ => Err(format!("{} cannot be parsed into type annotation.", s)),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    let config_path = opts.config.map(|v| PathBuf::from(v));
    let (cfg, mut lib) = load_config(config_path)?;
    match opts.subcmd {
        SubCommand::Info(opt) => do_info(opt, cfg, lib),
        SubCommand::Add(opt) => do_add(opt, cfg, &mut lib),
        SubCommand::Create(opt) => do_create(opt, cfg, &mut lib),
        SubCommand::Clean => {
            #[cfg(debug_assertions)]
            {
                drop(lib);
                purge_library(&cfg);
                recreate(&cfg);
            }
            Ok(())
        }
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

fn is_valid_media_type(v: &str) -> Result<(), String> {
    let k = MediaType::from_str(v.trim()).map_err(|_| "Unsupported Media Type.".to_string())?;
    match k {
        MediaType::Other => {
            if v.trim().to_ascii_lowercase() != "other" {
                Err("Not a valid Media Type.".to_string())
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}
