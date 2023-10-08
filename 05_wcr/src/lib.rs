use clap::{App, Arg};
use std::{
    error::Error,
    fs::File,
    io::{stdin, BufRead, BufReader},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("wcr")
        .version("0.1.0")
        .author("Denilson Bro")
        .about("Rust wc")
        .arg(
            Arg::with_name("files")
                .value_name("FILES")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("lines")
                .value_name("LINES")
                .short("l")
                .long("lines")
                .help("Count lines")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("bytes")
                .value_name("BYTES")
                .short("c")
                .long("bytes")
                .help("Count bytes")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("words")
                .value_name("WORDS")
                .short("w")
                .long("words")
                .help("Count words")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("chars")
                .value_name("CHARS")
                .short("m")
                .long("chars")
                .help("Count chars")
                .takes_value(false)
                .conflicts_with("bytes"),
        )
        .get_matches();

    let mut lines = matches.is_present("lines");
    let mut words = matches.is_present("words");
    let mut bytes = matches.is_present("bytes");
    let chars = matches.is_present("chars");

    // if [lines, words, bytes, chars].iter().all(|v| v == &false) {
    if [lines, words, bytes, chars].iter().all(|v| !v) {
        lines = true;
        words = true;
        bytes = true;
    }

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines,
        words,
        bytes,
        chars,
    })
}

fn format_field(value: usize, show: bool) -> String {
    if show {
        format!("{:>8}", value)
    } else {
        "".to_string()
    }
}

pub fn run(config: Config) -> MyResult<()> {
    let mut total_lines = 0;
    let mut total_words = 0;
    let mut total_bytes = 0;
    let mut total_chars = 0;

    for filename in &config.files {
        match open(&filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => {
                let fileinfo = count(file).unwrap();
                println!(
                    "{}{}{}{}{}",
                    format_field(fileinfo.num_lines, config.lines),
                    format_field(fileinfo.num_words, config.words),
                    format_field(fileinfo.num_chars, config.chars),
                    format_field(fileinfo.num_bytes, config.bytes),
                    if filename == "-" {
                        "".to_string()
                    } else {
                        format!(" {}", filename)
                    }
                );
                total_lines += fileinfo.num_lines;
                total_words += fileinfo.num_words;
                total_bytes += fileinfo.num_bytes;
                total_chars += fileinfo.num_chars;
            }
        }
    }
    if config.files.len() > 1 {
        println!(
            "{}{}{}{} total",
            format_field(total_lines, config.lines),
            format_field(total_words, config.words),
            format_field(total_chars, config.chars),
            format_field(total_bytes, config.bytes),
        )
    }

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

pub fn count(mut file: impl BufRead) -> MyResult<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;

    let mut buf = String::new();
    while let Ok(read_bytes) = file.read_line(&mut buf) {
        if read_bytes == 0 {
            break;
        }
        num_bytes += read_bytes;
        num_lines += 1;
        num_words += buf.split_whitespace().count();
        num_chars += buf.chars().count();
        buf.clear()
    }

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

#[cfg(test)]
mod tests {
    use crate::format_field;

    use super::{count, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_bytes: 48,
            num_chars: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }

    #[test]
    fn test_format_field() {
        assert_eq!(format_field(1, false), "");
        assert_eq!(format_field(3, true), "       3");
        assert_eq!(format_field(13, true), "      13");
    }
}
