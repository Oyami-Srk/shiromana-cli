use std::{boxed::Box, error::Error, fs, path};

use console::style;
use shiromana_rs::library::Library;

use crate::AppConfig;

pub fn create_library(config: &AppConfig) -> Result<Library, Box<dyn Error>> {
    println!(
        "{}",
        style("I am now creating Path and Library for you.")
            .blue()
            .bright()
    );
    dbg!(&config);
    fs::create_dir_all(path::Path::new(&config.library_path))?;
    Ok(Library::create(
        config.library_path.clone(),
        config.library_name.clone(),
        None,
        None,
    )?)
}

pub fn open_library(config: &AppConfig) -> Result<Library, Box<dyn Error>> {
    Ok(Library::open(
        config.library_path.clone()
            + if cfg!(target = "windows") { "\\" } else { "/" }
            + &config.library_name
            + ".mlib",
    )?)
}
