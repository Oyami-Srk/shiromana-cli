use crate::{Add, AppConfig, Create, Info};
use console::{style, Style, StyledObject};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Validator};
use humansize::{file_size_opts, FileSize};
use mime;
use shiromana_rs::library::{Library, LibrarySummary};
use shiromana_rs::media::{Media, MediaDetail, MediaType};
use shiromana_rs::misc::{Error as LibError, HashAlgo, Uuid};
use std::boxed::Box;
use std::convert::TryInto;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;
use tree_magic;
use url::{Host, ParseError, Position, Url};

lazy_static! {
    static ref DECO_LEFT_PAR_M: StyledObject<&'static str> = style("[").black().bright();
    static ref DECO_RIGHT_PAR_M: StyledObject<&'static str> = style("]").black().bright();
    static ref DECO_BRANCH: StyledObject<&'static str> = style("|-").black().bright();
    static ref STYLE_FIELD_NAME: Style = Style::new().yellow();
    static ref STYLE_FIELD_VALUE: Style = Style::new().blue().bright();
    static ref STYLE_ERROR: Style = Style::new().red().bright();
}

fn print_media(media: &Media, detailed: bool) {
    if detailed {
        println!(
            "{}: {}",
            STYLE_FIELD_NAME.apply_to("Media ID"),
            STYLE_FIELD_VALUE.apply_to(&media.id)
        );
        println!(
            "{}: {}",
            STYLE_FIELD_NAME.apply_to("Library UUID"),
            STYLE_FIELD_VALUE.apply_to(&media.library_uuid)
        );
        println!(
            "{}: {}",
            STYLE_FIELD_NAME.apply_to("Hash"),
            STYLE_FIELD_VALUE.apply_to(&media.hash)
        );
        println!(
            "{}: {}",
            STYLE_FIELD_NAME.apply_to("File Name"),
            STYLE_FIELD_VALUE.apply_to(&media.filename)
        );
        println!(
            "{}: {}",
            STYLE_FIELD_NAME.apply_to("File Path"),
            STYLE_FIELD_VALUE.apply_to(&media.filepath)
        );
        println!(
            "{}: {}",
            STYLE_FIELD_NAME.apply_to("File Size"),
            STYLE_FIELD_VALUE.apply_to(format!("{:2} KB", &media.filesize / 1024))
        );
        println!(
            "{}: {}",
            STYLE_FIELD_NAME.apply_to("Media Type"),
            STYLE_FIELD_VALUE.apply_to(&media.kind.to_string())
        );
        println!(
            "{}: {}",
            STYLE_FIELD_NAME.apply_to("Add Time"),
            STYLE_FIELD_VALUE.apply_to(&media.time_add.to_string())
        );
        if let Some(v) = &media.caption {
            println!(
                "{}: {}",
                STYLE_FIELD_NAME.apply_to("Caption"),
                STYLE_FIELD_VALUE.apply_to(v)
            );
        }
        if let Some(v) = &media.sub_kind {
            println!(
                "{}: {}",
                STYLE_FIELD_NAME.apply_to("Sub Type"),
                STYLE_FIELD_VALUE.apply_to(v)
            );
        }
        if let Some(v) = &media.kind_addition {
            println!(
                "{}: {}",
                STYLE_FIELD_NAME.apply_to("Type Addition"),
                STYLE_FIELD_VALUE.apply_to(v)
            );
        }
        if media.series_uuid.len() != 0 {
            println!(
                "{}:\n{}",
                STYLE_FIELD_NAME.apply_to("Series UUID"),
                STYLE_FIELD_VALUE.apply_to(
                    &media
                        .series_uuid
                        .iter()
                        .map(|u| { "    ".to_string() + &u.to_string() })
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            );
        }
        if let Some(v) = &media.comment {
            println!(
                "{}: {}",
                STYLE_FIELD_NAME.apply_to("Comment"),
                STYLE_FIELD_VALUE.apply_to(v)
            );
        }
        if let Some(v) = &media.detail {
            println!(
                "{}:\n{}",
                STYLE_FIELD_NAME.apply_to("Details"),
                format!("{}", v)
                    .split("\n")
                    .map(|v| "    ".to_string() + v)
                    .collect::<Vec<String>>()
                    .join("\n")
            );
        }
    } else {
        let decorator_style = Style::new().cyan().bright();
        let value_style = Style::new().blue();
        let filename_style = Style::new().yellow();
        println!(
            "{}{}{} {}{}{} {} - {}",
            decorator_style.apply_to("["),
            value_style.apply_to(media.id),
            decorator_style.apply_to("]"),
            decorator_style.apply_to("["),
            value_style.apply_to(media.kind.to_string()),
            decorator_style.apply_to("]"),
            filename_style.apply_to(&media.filename),
            value_style.apply_to(format!("{:2} KB", media.filesize / 1024))
        );
    }
}

pub fn do_info(opt: Info, _cfg: AppConfig, lib: Library) -> Result<(), Box<dyn Error>> {
    let print_library_info = || {
        println!(
            "{}: {}",
            STYLE_FIELD_NAME.apply_to("Library name"),
            STYLE_FIELD_VALUE.apply_to(lib.get_library_name())
        );
        match lib.get_master_name() {
            Some(v) => println!(
                "{}: {}",
                STYLE_FIELD_NAME.apply_to("Master name"),
                STYLE_FIELD_VALUE.apply_to(v)
            ),
            None => (),
        };
        println!(
            "{}: {}",
            STYLE_FIELD_NAME.apply_to("UUID"),
            STYLE_FIELD_VALUE.apply_to(lib.uuid.to_string())
        );
        println!(
            "{}: {}",
            STYLE_FIELD_NAME.apply_to("Path"),
            STYLE_FIELD_VALUE.apply_to(lib.get_path())
        );
        println!(
            "{}: {}",
            STYLE_FIELD_NAME.apply_to("Schema"),
            STYLE_FIELD_VALUE.apply_to(lib.get_schema())
        );
        let summary = lib.get_summary();
        println!("{}", STYLE_FIELD_NAME.apply_to("Library Summary"));
        println!(
            "    {} {}: {}",
            *DECO_BRANCH,
            STYLE_FIELD_NAME.apply_to("Media count"),
            STYLE_FIELD_VALUE.apply_to(format!("{}", summary.media_count))
        );
        println!(
            "    {} {}: {}",
            *DECO_BRANCH,
            STYLE_FIELD_NAME.apply_to("Series count"),
            STYLE_FIELD_VALUE.apply_to(format!("{}", summary.series_count))
        );
        println!(
            "    {} {}: {}",
            *DECO_BRANCH,
            STYLE_FIELD_NAME.apply_to("Media size"),
            STYLE_FIELD_VALUE.apply_to(format!(
                "{}",
                summary
                    .media_size
                    .file_size(file_size_opts::CONVENTIONAL)
                    .unwrap()
            )),
        );
    };
    let get_media = |query_string: String| {
        // first try ID
        match query_string.parse() {
            Ok(v) => {
                let media = lib.get_media(v);
                if media.is_ok() {
                    return (vec![media.unwrap()], query_string);
                }
            }
            Err(_) => (),
        };
        // then try Hash
        if query_string.trim().len() == lib.get_hash_size() * 2 {
            let ids = lib.query_media(&format!("hash = '{}'", query_string.trim()));
            if ids.is_ok() {
                return (
                    vec![lib.get_media(*ids.unwrap().first().unwrap()).unwrap()],
                    query_string,
                );
            }
        }
        // last try file name
        let media = lib.get_media_by_filename(query_string.trim().to_string());
        match media {
            Ok(v) => (
                v.iter().map(|id| lib.get_media(*id).unwrap()).collect(),
                query_string,
            ),
            Err(_) => (vec![], query_string),
        }
    };

    match opt.media {
        Some(v) => {
            let (media, query_string) = get_media(v);
            if media.is_empty() {
                println!(
                    "{} {}",
                    STYLE_ERROR.apply_to("Cannot acquire any media via: "),
                    STYLE_FIELD_VALUE.apply_to(query_string)
                )
            } else {
                for media in media.iter() {
                    print_media(media, opt.detail);
                }
            }
        }
        None => print_library_info(),
    };
    Ok(())
}

fn add_one_media(
    lib: &mut Library,
    file: PathBuf,
    kind: Option<MediaType>,
    title: Option<String>,
    comment: Option<String>,
) -> Result<u64, LibError> {
    let kind = kind.clone().unwrap_or_else(|| {
        let mime_str = tree_magic::from_filepath(file.as_path());
        let mime_str = mime_str.split("/").collect::<Vec<&str>>();
        let mime_str = mime_str.first().unwrap();
        match *mime_str {
            "image" => MediaType::Image,
            "audio" => MediaType::Audio,
            "video" => MediaType::Video,
            "text" => MediaType::Text,
            _ => MediaType::Other,
        }
    });
    let id = lib.add_media(
        file.to_str().unwrap().to_string(),
        kind.clone(),
        None,
        None,
        title,
        comment,
    )?;
    println!(
        "{}: {} {}{}{} {}{}{}",
        STYLE_FIELD_NAME.apply_to("Successfully Added Media"),
        STYLE_FIELD_VALUE.apply_to(
            file.file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
        ),
        *DECO_LEFT_PAR_M,
        STYLE_FIELD_VALUE.apply_to(id),
        *DECO_RIGHT_PAR_M,
        *DECO_LEFT_PAR_M,
        STYLE_FIELD_VALUE.apply_to(kind.to_string()),
        *DECO_RIGHT_PAR_M,
    );
    Ok(id)
}

fn parse_input_file(input: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let lines = std::fs::read_to_string(input)?;
    let lines = lines.lines();
    let lines = lines.map(|v| {
        if &v[0..7] == "file://" {
            Url::parse(v).unwrap().path().to_string()
        } else {
            v.to_string()
        }
    });
    let not_exists: Vec<String> = lines
        .clone()
        .filter(|v| !PathBuf::from(v).is_file())
        .collect();
    if !not_exists.is_empty() {
        Err({ not_exists.join(",") + " are not existed or not a file." }.into())
    } else {
        Ok(lines.map(|v| PathBuf::from(v)).collect())
    }
}

pub fn do_add(opt: Add, _cfg: AppConfig, lib: &mut Library) -> Result<(), Box<dyn Error>> {
    let files = if let Some(input) = &opt.input {
        parse_input_file(&input)?
    } else {
        opt.file.clone()
    };
    let (title, comment) = if files.len() == 1 {
        (opt.title.clone(), opt.comment.clone())
    } else {
        (None, None)
    };

    let ids: Vec<Option<u64>> = files
        .iter()
        .map(|f| {
            match add_one_media(
                lib,
                f.clone(),
                opt._type.clone(),
                title.clone(),
                comment.clone(),
            ) {
                Err(e) => {
                    if let LibError::AlreadyExists(s) = e {
                        let id = lib
                            .query_media(&format!("hash = '{}'", s))
                            .unwrap_or_else(|v| {
                                println!(
                                    "{}: {}, {}: {}",
                                    STYLE_ERROR
                                        .apply_to("Error at querying media via Hash should exists"),
                                    STYLE_FIELD_VALUE.apply_to(s),
                                    STYLE_ERROR.apply_to("Due to"),
                                    STYLE_FIELD_VALUE.apply_to(v.to_string())
                                );
                                vec![]
                            })
                            .first()
                            .map(|v| *v);
                        if let Some(id) = id {
                            let m = lib.get_media(id).unwrap();
                            println!(
                                "{}: {} {}{}{} {}{}{}",
                                STYLE_FIELD_NAME.apply_to("Existed Media Found"),
                                STYLE_FIELD_VALUE.apply_to(m.filename),
                                *DECO_LEFT_PAR_M,
                                STYLE_FIELD_VALUE.apply_to(id),
                                *DECO_RIGHT_PAR_M,
                                *DECO_LEFT_PAR_M,
                                STYLE_FIELD_VALUE.apply_to(m.kind.to_string()),
                                *DECO_RIGHT_PAR_M,
                            );
                        }
                        id
                    } else {
                        println!(
                            "{}: {}",
                            STYLE_ERROR.apply_to("Error when trying add media"),
                            STYLE_FIELD_VALUE.apply_to(e.to_string())
                        );
                        None
                    }
                }
                Ok(id) => Some(id),
            }
        })
        .collect();
    if opt.sorted && ids.iter().any(|v| v.is_none()) {
        println!("{}", STYLE_FIELD_VALUE.apply_to("There is some media cannot be added while trying to add it to sorted series. This may break the sort."));
        return Ok(());
    }

    let series = if let Some(uuid) = opt.series {
        Some(uuid)
    } else if let Some(name) = opt.new_series {
        Some(lib.create_series(
            if lib.get_series_by_name(name.clone()).is_ok() {
                let theme = ColorfulTheme {
                    values_style: Style::new().yellow().dim(),
                    ..ColorfulTheme::default()
                };
                let r = Input::with_theme(&theme)
                    .with_prompt("Series name")
                    .validate_with(|input: &String| -> Result<(), &str> {
                        if lib.get_series_by_name(input.clone()).is_ok() {
                            Err("This is name is existed. Choose another one please.")
                        } else {
                            Ok(())
                        }
                    })
                    .interact_text()?;
                r
            } else {
                name
            },
            None,
        )?)
    } else {
        None
    };

    if let Some(uuid) = series {
        let mut ids = ids;
        ids.retain(|c| c.is_some());
        dbg!(ids.clone());
        for (i, id) in ids.iter().enumerate() {
            lib.add_to_series(id.unwrap(), &uuid, None, !opt.sorted)?;
        }
        println!(
            "Successfully Added {} Medias to Series {}.",
            ids.len(),
            uuid
        );
    }
    Ok(())
}

pub fn do_create(opt: Create, _cfg: AppConfig, lib: &mut Library) -> Result<(), Box<dyn Error>> {
    let uuid = lib.create_series(opt.title.clone(), opt.comment)?;
    if opt.uuid_only {
        println!("{}", uuid);
    } else {
        println!(
            "{}: {}{}{} {}{}{}",
            STYLE_FIELD_NAME.apply_to("Successfully created series"),
            *DECO_LEFT_PAR_M,
            STYLE_FIELD_NAME.apply_to(opt.title),
            *DECO_RIGHT_PAR_M,
            *DECO_LEFT_PAR_M,
            STYLE_FIELD_NAME.apply_to(uuid),
            *DECO_RIGHT_PAR_M,
        );
    }
    Ok(())
}
