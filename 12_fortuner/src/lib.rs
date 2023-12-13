use std::{error::Error, path::PathBuf, vec};

use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("fortuner")
        .version("0.1.0")
        .author("Denilson <denilson020898@gmail.com>")
        .about("Rust fortune")
        .arg(
            Arg::with_name("files")
                .value_name("FILES")
                .help("Input files or directories")
                .multiple(true)
                .required(true),
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .long("insensitive")
                .help("Case-insensitive pattern matching"),
        )
        .arg(
            Arg::with_name("pattern")
                .short("m")
                .long("pattern")
                .value_name("PATTERN")
                .help("Pattern"),
        )
        .arg(
            Arg::with_name("seed")
                .short("s")
                .long("seed")
                .value_name("SEED")
                .help("Random seed"),
        )
        .get_matches();

    let seed = matches.value_of("seed").map(parse_u64).transpose()?;

    // # control flow is too complicated, using transpose is cooler
    //
    // let pattern = if let Some(pattern) = matches.value_of("pattern") {
    //     let rgx_pattern = RegexBuilder::new(pattern)
    //         .case_insensitive(matches.is_present("insensitive"))
    //         .build()
    //         .map_err(|_| format!("Invalid --pattern \"{}\"", pattern))?;
    //     Some(rgx_pattern)
    // } else {
    //     None
    // };

    let pattern = matches
        .value_of("pattern")
        .map(|val| {
            RegexBuilder::new(val)
                .case_insensitive(matches.is_present("insensitive"))
                .build()
                .map_err(|_| format!("Invalid --pattern \"{}\"", val))
        })
        .transpose()?;

    Ok(Config {
        sources: matches.values_of_lossy("files").unwrap(),
        pattern,
        seed,
    })
}

fn parse_u64(val: &str) -> MyResult<u64> {
    val.parse::<u64>()
        .map_err(|_| format!("\"{}\" not a valid integer", val).into())
}

pub fn run(config: Config) -> MyResult<()> {
    let files = find_files(&config.sources)?;
    println!("{:#?}", files);
    Ok(())
}

fn find_files(paths: &[String]) -> MyResult<Vec<PathBuf>> {
    // let mut paths_buf = paths
    //     .into_iter()
    //     .map(PathBuf::from)
    //     .filter(|f| f.try_exists().unwrap())
    //     .collect::<Vec<PathBuf>>();

    let mut paths_buf = Vec::new();
    for path in paths {
        let path_buf = PathBuf::from(path);
        path_buf.try_exists()?;
        paths_buf.push(path_buf);
    }

    paths_buf.sort();
    paths_buf.dedup();

    // for path in paths_buf.iter() {
    //     if !path.exists() {
    //         return Err(From::from(format!(
    //             "Path to \"{}\" does not exist",
    //             path.to_string_lossy()
    //         )));
    //     }
    // }

    let paths_buf = paths_buf
        .iter()
        .flat_map(|path| {
            WalkDir::new(path)
                // .max_depth(1)
                .sort_by_file_name()
                .into_iter()
                .filter_map(|e| match e {
                    Ok(entry) => {
                        let entry_path = entry.path();

                        if entry_path.is_dir() {
                            None
                        } else {
                            match entry_path.extension() {
                                Some(ext) if ext == "dat" => None,
                                _ => Some(entry.into_path()),
                            }
                        }
                    }
                    _ => None,
                })
        })
        .collect::<Vec<PathBuf>>();
    return Ok(paths_buf);
}

#[cfg(test)]
mod tests {
    use crate::{find_files, parse_u64};

    #[test]
    fn test_parse_u64() {
        let res = parse_u64("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "\"a\" not a valid integer");

        let res = parse_u64("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = parse_u64("4");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 4);
    }

    #[test]
    fn test_find_files() {
        // success call, able to find
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // failed call
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // finds all input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());

        // check number and order of files
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // test multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);

        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }
}
