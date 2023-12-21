use std::os::unix::fs::PermissionsExt;
use std::os::unix::prelude::MetadataExt;
use std::{error::Error, fmt::format, path::PathBuf};

use chrono::{DateTime, Local};
use clap::{App, Arg};
use tabular::{Row, Table};
use users::{get_user_by_uid, get_group_by_gid};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    long: bool,
    show_hidden: bool,
}

fn find_files(paths: &[String], show_hidden: bool) -> MyResult<Vec<PathBuf>> {
    let mut result = Vec::new();

    for path in paths {
        match std::fs::metadata(path) {
            Err(e) => eprintln!("{}: {}", path, e),
            Ok(md) => {
                if md.is_file() {
                    let pathbuf = PathBuf::from(path);
                    result.push(pathbuf);
                } else {
                    let dirs = std::fs::read_dir(path)?;
                    for dir in dirs {
                        let pathbuf = dir?.path();
                        let is_hidden = pathbuf.file_name().map_or(false, |filename| {
                            filename.to_string_lossy().starts_with(".")
                        });

                        if !show_hidden && is_hidden {
                            continue;
                        }
                        result.push(pathbuf);
                    }
                }
            }
        }
    }

    Ok(result)
}

fn format_mode(mode: u32) -> String {
    let full_access = "rwxrwxrwx".to_string();
    let mode_as_chars: Vec<_> = format!(
        "{:0>3b}{:0>3b}{:0>3b}",
        (mode >> 6) & 0b111,
        (mode >> 3) & 0b111,
        mode & 0b111
    )
    .chars()
    .collect();

    let mut result = "".to_string();

    for (full_ref, input_mode) in full_access.chars().zip(mode_as_chars) {
        if input_mode == '1' {
            result.push(full_ref)
        } else {
            result.push('-')
        }
    }

    result
}

fn format_output(paths: &[PathBuf]) -> MyResult<String> {
    let fmt = "{:<}{:<}  {:>}  {:<}  {:<}  {:>}  {:<}  {:<}";
    let mut table = Table::new(fmt);

    for path in paths {
        let md = path.metadata()?;

        let modified: DateTime<Local> = DateTime::from(md.modified()?);
        let uid = md.uid();
        let user = get_user_by_uid(uid).map(|u| u.name().to_string_lossy().into_owned()).unwrap_or_else(||uid.to_string());

        let gid = md.gid();
        let group = get_group_by_gid(gid).map(|g| g.name().to_string_lossy().into_owned()).unwrap_or_else(||gid.to_string());

        table.add_row(
            Row::new()
                .with_cell(if md.is_dir() { "d" } else { "-" }) //  1 "d" or "-"
                .with_cell(format_mode(md.permissions().mode())) //  2 permission
                .with_cell(md.nlink()) //  3 num of symlinks
                .with_cell(user) //  4 username
                .with_cell(group) //  5 group name
                .with_cell(md.size().to_string()) //  6 size in bytes
                .with_cell(modified.format("%b %d %y %H:%M")) //  7 last modification
                .with_cell(path.display()), // 8 path
        );
    }
    Ok(format!("{}", table))
}

// Owner Read  Write Execute
// User  0o400 0o200 0o100
// Group 0o040 0o020 0o010
// Other 0o004 0o002 0o001

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::format_output;

    use super::format_mode;

    use super::find_files;

    #[test]
    fn test_find_files_nonhidden_1_dir() {
        // find all non-hidden file entries in 1 directory
        let res = find_files(&["tests/inputs".to_string()], false);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        )
    }

    #[test]
    fn test_find_files_all_entries_1_dir() {
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        )
    }

    #[test]
    fn test_find_files_hidden_file_is_shown_if_targetted_directly() {
        let res = find_files(&["tests/inputs/.hidden".to_string()], false);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(filenames, ["tests/inputs/.hidden",])
    }

    #[test]
    fn test_find_files_multiple_path() {
        let res = find_files(
            &[
                "tests/inputs/bustle.txt".to_string(),
                "tests/inputs/dir".to_string(),
            ],
            false,
        );
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            ["tests/inputs/bustle.txt", "tests/inputs/dir/spiders.txt",]
        )
    }

    #[test]
    fn test_format_mode() {
        assert_eq!(format_mode(0o775), "rwxrwxr-x");
        assert_eq!(format_mode(0o421), "r---w---x");
        assert_eq!(format_mode(0o777), "rwxrwxrwx");
        assert_eq!(format_mode(0o000), "---------");
    }

    fn long_match(
        line: &str,
        expected_name: &str,
        expected_perms: &str,
        expected_size: Option<&str>,
    ) {
        let parts: Vec<_> = line.split_whitespace().collect();
        assert!(parts.len() > 0 && parts.len() <= 10);
        let perms = parts.get(0).unwrap();
        assert_eq!(perms, &expected_perms);

        if let Some(size) = expected_size {
            let file_size = parts.get(4).unwrap();
            assert_eq!(file_size, &size);
        }

        let display_name = parts.last().unwrap();
        assert_eq!(display_name, &expected_name);
    }

    #[test]
    fn test_format_output_one() {
        let bustle_path = "tests/inputs/bustle.txt";
        let bustle = PathBuf::from(bustle_path);

        let res = format_output(&[bustle]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let lines: Vec<&str> = out.split("\n").filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), 1);

        let line1 = lines.first().unwrap();
        long_match(&line1, bustle_path, "-rw-r--r--", Some("193"));
    }

    #[test]
    fn test_format_output_two() {
        let res = format_output(&[
            PathBuf::from("tests/inputs/dir"),
            PathBuf::from("tests/inputs/empty.txt"),
        ]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let mut lines: Vec<&str> = out.split("\n").filter(|s| !s.is_empty()).collect();
        lines.sort();
        assert_eq!(lines.len(), 2);

        let empty_line = lines.remove(0);
        long_match(
            &empty_line,
            "tests/inputs/empty.txt",
            "-rw-r--r--",
            Some("0"),
        );

        let line_dir = lines.remove(0);
        long_match(&line_dir, "tests/inputs/dir", "drwxr-xr-x", None);
    }
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("lsr")
        .version("0.1.0")
        .author("Denilson <denilson020898@gmail.com>")
        .about("Rust ls")
        .arg(
            Arg::with_name("long")
                .help("Long listing")
                .short("l")
                .long("long"),
        )
        .arg(
            Arg::with_name("all")
                .help("Show all files")
                .short("a")
                .long("all"),
        )
        .arg(
            Arg::with_name("paths")
                .help("Files and/or directories")
                .value_name("PATH")
                .takes_value(false)
                .multiple(true)
                .default_value("."),
        )
        .get_matches();

    Ok(Config {
        paths: matches.values_of_lossy("paths").unwrap(),
        long: matches.is_present("long"),
        show_hidden: matches.is_present("all"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let paths = find_files(&config.paths, config.show_hidden)?;
    if config.long {
        println!("{}", format_output(&paths)?);
    } else {
        for path in paths {
            println!("{}", path.display());
        }
    }
    Ok(())
}
