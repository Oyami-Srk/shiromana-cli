use crate::{Add, AppConfig, Info};
use console::{style, Style};
use shiromana_rs::library::{Library, LibrarySummary};
use shiromana_rs::media::{Media, MediaDetail, MediaType};
use shiromana_rs::misc::{HashAlgo, Uuid};
use std::boxed::Box;
use std::convert::TryInto;
use std::error::Error;
use std::str::FromStr;

fn print_media(media: &Media, detailed: bool) {
    if detailed {
        let field_name_style = Style::new().yellow();
        let field_value_style = Style::new().blue().bright();
        let field_decorator_style = Style::new().black().bright();
        println!(
            "{}: {}",
            field_name_style.apply_to("Media ID"),
            field_value_style.apply_to(&media.id)
        );
        println!(
            "{}: {}",
            field_name_style.apply_to("Library UUID"),
            field_value_style.apply_to(&media.library_uuid)
        );
        println!(
            "{}: {}",
            field_name_style.apply_to("Hash"),
            field_value_style.apply_to(&media.hash)
        );
        println!(
            "{}: {}",
            field_name_style.apply_to("File Name"),
            field_value_style.apply_to(&media.filename)
        );
        println!(
            "{}: {}",
            field_name_style.apply_to("File Path"),
            field_value_style.apply_to(&media.filepath)
        );
        println!(
            "{}: {}",
            field_name_style.apply_to("File Size"),
            field_value_style.apply_to(format!("{:2} KB", &media.filesize / 1024))
        );
        println!(
            "{}: {}",
            field_name_style.apply_to("Media Type"),
            field_value_style.apply_to(&media.kind.to_string())
        );
        println!(
            "{}: {}",
            field_name_style.apply_to("Add Time"),
            field_value_style.apply_to(&media.time_add.to_string())
        );
        if let Some(v) = &media.caption {
            println!(
                "{}: {}",
                field_name_style.apply_to("Caption"),
                field_value_style.apply_to(v)
            );
        }
        if let Some(v) = &media.sub_kind {
            println!(
                "{}: {}",
                field_name_style.apply_to("Sub Type"),
                field_value_style.apply_to(v)
            );
        }
        if let Some(v) = &media.kind_addition {
            println!(
                "{}: {}",
                field_name_style.apply_to("Type Addition"),
                field_value_style.apply_to(v)
            );
        }
        if let Some(v) = &media.series_uuid {
            println!(
                "{}: {}",
                field_name_style.apply_to("Series UUID"),
                field_value_style.apply_to(v)
            );
        }
        if let Some(v) = &media.series_no {
            println!(
                "{}: #{}",
                field_name_style.apply_to("Series No"),
                field_value_style.apply_to(v)
            );
        }
        if let Some(v) = &media.comment {
            println!(
                "{}: {}",
                field_name_style.apply_to("Comment"),
                field_value_style.apply_to(v)
            );
        }
        if let Some(v) = &media.detail {
            println!(
                "{}:\n{}",
                field_name_style.apply_to("Details"),
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
    let field_name_style = Style::new().yellow();
    let field_value_style = Style::new().blue().bright();
    let field_decorator_style = Style::new().black().bright();
    let print_library_info = || {
        println!(
            "{}: {}",
            field_name_style.apply_to("Library name"),
            field_value_style.apply_to(lib.get_library_name())
        );
        match lib.get_master_name() {
            Some(v) => println!(
                "{}: {}",
                field_name_style.apply_to("Master name"),
                field_value_style.apply_to(v)
            ),
            None => (),
        };
        println!(
            "{}: {}",
            field_name_style.apply_to("UUID"),
            field_value_style.apply_to(lib.uuid.to_string())
        );
        println!(
            "{}: {}",
            field_name_style.apply_to("Path"),
            field_value_style.apply_to(lib.get_path())
        );
        println!(
            "{}: {}",
            field_name_style.apply_to("Schema"),
            field_value_style.apply_to(lib.get_schema())
        );
        let summary = lib.get_summary();
        println!("{}", field_name_style.apply_to("Library Summary"));
        println!(
            "    {} {}: {}",
            field_decorator_style.apply_to("|-"),
            field_name_style.apply_to("Media count"),
            field_value_style.apply_to(format!("{}", summary.media_count))
        );
        println!(
            "    {} {}: {}",
            field_decorator_style.apply_to("|-"),
            field_name_style.apply_to("Series count"),
            field_value_style.apply_to(format!("{}", summary.series_count))
        );
        println!(
            "    {} {}: {}",
            field_decorator_style.apply_to("|-"),
            field_name_style.apply_to("Media size"),
            field_value_style.apply_to(format!("{} KB", summary.media_size)),
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
                let info_style = Style::new().red().bright();
                let info_value = Style::new().yellow();
                println!(
                    "{} {}",
                    info_style.apply_to("Cannot acquire any media via: "),
                    info_value.apply_to(query_string)
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

pub fn do_add(opt: Add, _cfg: AppConfig, lib: Library) -> Result<(), Box<dyn Error>> {
    Ok(())
}
