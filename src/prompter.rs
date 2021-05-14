use std::error::Error;
use std::path::Path;

use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input};

pub fn ask_for_location(is_dir: bool, default: String) -> Result<String, Box<dyn Error>> {
    let theme = ColorfulTheme {
        values_style: Style::new().yellow().dim(),
        ..ColorfulTheme::default()
    };

    Ok(loop {
        let location = Input::with_theme(&theme)
            .with_prompt("Library Path")
            .with_initial_text(default.clone())
            .default(default.clone())
            .validate_with(move |input: &String| -> Result<(), &str> {
                if {
                    if cfg!(target_os = "windows") {
                        vec!["*", ":", "?", ">", "<", "|", "\""]
                    } else if cfg!(target_os = "macos") {
                        vec![":"]
                    } else if cfg!(target_os = "linux") {
                        vec![]
                    } else {
                        unimplemented!(
                            "This program only support for macOS and Linux or maybe Windows later."
                        );
                    }
                }
                .iter()
                .any(|item| input.contains(item))
                {
                    return Err("Path contains invalid characters.");
                }
                let path = Path::new(input);
                if is_dir && path.is_file() {
                    return Err("Path is already existed as a file.");
                } else if !is_dir && path.is_dir() {
                    return Err("File is already existed as a dir.");
                }
                Ok(())
            })
            .interact_text()?;
        let mut p = Path::new(&location);
        let mut pv: Vec<&str> = if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
            location.split("/").collect()
        } else if cfg!(target = "windows") {
            unimplemented!("This program only support for macOS and Linux YET.");
        } else {
            unimplemented!("This program only support for macOS and Linux or maybe Windows later.");
        };
        let mut npv: Vec<&str> = vec![];
        while !p.exists() {
            p = match p.parent() {
                Some(v) => v,
                None => panic!("Root or current dir not even exists??? WHAT?"),
            };
            npv.push(pv.pop().unwrap());
        }
        let exist_path: String = pv.join("/");
        let non_exists_path: String = npv.join("/");
        if npv.is_empty() {
            break Path::new(&location)
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
        }
        if Confirm::with_theme(&theme)
            .default(true)
            .with_prompt(format!(
                "I will create {} on the existed folder {} for you. Would that be OK?",
                non_exists_path, exist_path
            ))
            .interact()?
        {
            break Path::new(&location)
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
        }
    })
}

pub fn ask_for_library_name(default: String) -> Result<String, Box<dyn Error>> {
    let theme = ColorfulTheme {
        values_style: Style::new().yellow().dim(),
        ..ColorfulTheme::default()
    };

    let r = Input::with_theme(&theme)
        .with_prompt("Library Path")
        .with_initial_text(default.clone())
        .default(default.clone())
        .validate_with(move |input: &String| -> Result<(), &str> {
            if {
                if cfg!(target_os = "windows") {
                    vec!["*", ":", "?", ">", "<", "|", "\""]
                } else if cfg!(target_os = "macos") {
                    vec![":"]
                } else if cfg!(target_os = "linux") {
                    vec![]
                } else {
                    unimplemented!(
                        "This program only support for macOS and Linux or maybe Windows later."
                    );
                }
            }
            .iter()
            .any(|item| input.contains(item))
            {
                return Err("Library name contains invalid characters.");
            }
            Ok(())
        })
        .interact_text()?;
    Ok(r)
}
