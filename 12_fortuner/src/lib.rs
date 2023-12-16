use std::{
    error::Error,
    ffi::OsStr,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    vec,
};

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

#[derive(Debug)]
struct Fortune {
    sources: String,
    text: String,
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
    let mut paths_buf = Vec::new();
    for path in paths {
        match std::fs::metadata(path) {
            Err(e) => return Err(format!("{}: {}", path, e).into()),
            Ok(_) => paths_buf.extend(
                WalkDir::new(path)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| {
                        e.file_type().is_file() && e.path().extension() != Some(OsStr::new("dat"))
                    })
                    .map(|e| e.path().into()),
            ),
        };
    }

    paths_buf.sort();
    paths_buf.dedup();

    return Ok(paths_buf);
}

fn read_fortunes(paths: &[PathBuf]) -> MyResult<Vec<Fortune>> {
    let mut result: Vec<Fortune> = Vec::new();

    for path in paths {

        let file = File::open(path)?;
        let mut buf_reader = BufReader::new(file);

        let mut buf = Vec::new();
        loop {
            let sources = path.to_string_lossy().to_string();
            let bytes_read = buf_reader.read_until(b'%', &mut buf)?;
            if bytes_read == 0 {
                break;
            }
            let raw_text = String::from_utf8_lossy(&buf).into_owned();
            let text: String = raw_text.replace("%", "").trim().into();

            if !text.is_empty() {
                let fortune = Fortune {
                    sources,
                    text,
                };
                result.push(fortune);
            }
            buf.clear();
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{find_files, parse_u64, read_fortunes};

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
    fn test_find_file_one_file_pass() {
        // success call, able to find
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );
    }

    #[test]
    fn test_find_file_does_not_exists_should_fail() {
        // failed call
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());
    }

    #[test]
    fn test_find_file_directory_without_dat_extension_pass() {
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
    }

    #[test]
    fn test_find_file_multiple_file_path_pass() {
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

    #[test]
    fn test_read_fortunes_one_input_file() {
        // one input file
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());

        if let Ok(fortunes) = res {
            // correct number and sorting
            assert_eq!(fortunes.len(), 6);

            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
                A. Collared greens."
            );

            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
                A: A bad idea (bad-eye deer)."
            );
        }
    }

    #[test]
    fn test_read_fortunes_multiple_input_file() {
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }
}
