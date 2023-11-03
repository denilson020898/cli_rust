use std::error::Error;

use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use walkdir::{DirEntry, WalkDir};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    pattern: Regex,
    files: Vec<String>,
    recursive: bool,
    count: bool,
    invert_match: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("grepr")
        .version("0.1.0")
        .author("Denilson Bro <denilson020898@gmail.com>")
        .about("Rust grep")
        .arg(
            Arg::with_name("pattern")
                .help("Search pattern")
                .value_name("PATTERN")
                .required(true),
        )
        .arg(
            Arg::with_name("files")
                .help("Input file(s)")
                .value_name("FILE")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Count occurrences")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .long("insensitive")
                .help("Case-insensitive")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("invert-match")
                .short("v")
                .long("invert-match")
                .help("Invert match")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("recursive")
                .short("r")
                .long("recursive")
                .help("Recursive search")
                .takes_value(false),
        )
        .get_matches();

    let pattern = matches.value_of("pattern").unwrap();

    let pattern = RegexBuilder::new(pattern)
        .case_insensitive(matches.is_present("insensitive"))
        .build()
        .map_err(|_| format!("Invalid pattern \"{}\"", pattern))?;
    let files = matches.values_of_lossy("files").unwrap();

    Ok(Config {
        pattern,
        files,
        recursive: matches.is_present("recursive"),
        count: matches.is_present("count"),
        invert_match: matches.is_present("invert-match"),
    })
}

fn find_files(paths: &[String], recursive: bool) -> Vec<MyResult<String>> {
    let mut results = vec![];

    for path in paths {
        match path.as_str() {
            "-" => results.push(Ok(path.to_string())),
            _ => {
                let walks = WalkDir::new(path);
                let walks = if recursive { walks } else { walks.max_depth(0) };
                let walked_path = walks
                    .into_iter()
                    .filter_map(|e| match e {
                        Ok(e) => {
                            let is_dir = e.path().is_dir();
                            let path = e.path().display().to_string();
                            if !recursive && is_dir {
                                return Some(Err(format!("{} is a directory", path).into()));
                            } else if recursive && is_dir {
                                return None;
                            }
                            Some(Ok(path))
                        }
                        Err(e) => Some(Err(e.into())),
                    })
                    .collect::<Vec<_>>();

                results.extend(walked_path.into_iter())
            }
        }
    }

    return results;

    // paths
    //     .into_iter()
    //     .map(|path| {
    //         let walks = WalkDir::new(path);
    //         let walks = if recursive { walks } else { walks.max_depth(0) };
    //         walks.into_iter()
    //     })
    //     .flatten()
    //     .filter_map(|e| match e {
    //         Ok(e) => {
    //             let is_dir = e.path().is_dir();
    //             let path = e.path().display().to_string();
    //             if !recursive && is_dir {
    //                 return Some(Err(format!("{} is a directory", path).into()));
    //             } else if recursive && is_dir {
    //                 return None;
    //             }
    //             Some(Ok(path))
    //         }
    //         Err(e) => Some(Err(e.into())),
    //     })
    //     .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::find_files;
    use rand::{distributions::Alphanumeric, Rng};

    #[test]
    fn test_find_files() {
        // verify the function finds an existing file
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        // reject a directory without a recursive flag
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }

        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();

        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );

        let bad: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }
}

pub fn run(config: Config) -> MyResult<()> {
    println!("pattern \"{}\"", config.pattern);
    let entries = find_files(&config.files, config.recursive);
    for entry in entries {
        match entry {
            Ok(filename) => println!("field \"{}\"", filename),
            Err(e) => eprintln!("{}", e),
        }
    }
    Ok(())
}
