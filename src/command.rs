use crate::{Add, AppConfig, Info};
use console::{style, Style, StyledObject};
use mime;
use shiromana_rs::library::{Library, LibrarySummary};
use shiromana_rs::media::{Media, MediaDetail, MediaType};
use shiromana_rs::misc::{HashAlgo, Uuid};
use std::boxed::Box;
use std::convert::TryInto;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;
use tree_magic;

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
        if let Some(v) = &media.series_uuid {
            println!(
                "{}: {}",
                STYLE_FIELD_NAME.apply_to("Series UUID"),
                STYLE_FIELD_VALUE.apply_to(v)
            );
        }
        if let Some(v) = &media.series_no {
            println!(
                "{}: #{}",
                STYLE_FIELD_NAME.apply_to("Series No"),
                STYLE_FIELD_VALUE.apply_to(v)
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
            STYLE_FIELD_VALUE.apply_to(format!("{} KB", summary.media_size)),
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
) -> Result<u64, Box<dyn Error>> {
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

pub fn do_add(opt: Add, _cfg: AppConfig, lib: &mut Library) -> Result<(), Box<dyn Error>> {
    if opt.file.len() == 1 {
        add_one_media(
            lib,
            opt.file.first().unwrap().clone(),
            opt._type.clone(),
            opt.title.clone(),
            opt.comment.clone(),
        );
    } else {
        for f in opt.file {
            match add_one_media(lib, f, opt._type.clone(), None, None) {
                Err(e) => println!(
                    "{}: {}",
                    STYLE_ERROR.apply_to("Error when trying add media: "),
                    STYLE_FIELD_VALUE.apply_to(e.to_string())
                ),
                Ok(id) => (),
            }
        }
    }
    Ok(())
}
