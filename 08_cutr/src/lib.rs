use clap::{App, Arg};
use csv::{StringRecord, ReaderBuilder, WriterBuilder};
use regex::Regex;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    ops::{Deref, Index, Range},
    usize,
};

type MyResult<T> = Result<T, Box<dyn Error>>;
type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    delimiter: u8,
    extract: Extract,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("cutr")
        .version("0.1.0")
        .author("Denilson <denilson020898@gmail.com>")
        .about("Rust cut")
        .arg(
            Arg::with_name("files")
                .help("Input file(s)")
                .takes_value(false)
                .value_name("FILE")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("delim")
                .short("d")
                .long("delim")
                .help("Field delimiter")
                .takes_value(true)
                .value_name("DELIMETER")
                .default_value("\t"),
        )
        .arg(
            Arg::with_name("bytes")
                .short("b")
                .long("bytes")
                .help("Selected bytes")
                .takes_value(true)
                .value_name("BYTES")
                .conflicts_with("chars")
                .conflicts_with_all(&["fields", "chars"]),
        )
        .arg(
            Arg::with_name("chars")
                .short("c")
                .long("chars")
                .help("Selected characters")
                .takes_value(true)
                .value_name("CHARS")
                .conflicts_with_all(&["fields", "bytes"]),
        )
        .arg(
            Arg::with_name("fields")
                .short("f")
                .long("fields")
                .help("Selected fields")
                .takes_value(true)
                .value_name("FIELDS")
                .conflicts_with_all(&["bytes", "chars"]),
        )
        .get_matches();

    let files = matches.values_of_lossy("files").unwrap();
    let delimiter = matches.value_of_lossy("delim").unwrap();
    let delim_bytes = delimiter.as_bytes();
    if delim_bytes.len() != 1 {
        return Err(From::from(format!(
            "--delim \"{}\" must be a single byte",
            delimiter
        )));
    }

    let fields = matches.value_of("fields").map(parse_pos).transpose()?;
    let bytes = matches.value_of("bytes").map(parse_pos).transpose()?;
    let chars = matches.value_of("chars").map(parse_pos).transpose()?;

    let ranges = if let Some(field_range) = fields {
        Extract::Fields(field_range)
    } else if let Some(bytes_range) = bytes {
        Extract::Bytes(bytes_range)
    } else if let Some(chars_range) = chars {
        Extract::Chars(chars_range)
    } else {
        return Err("Must have --fields, --bytes, or --chars".into());
    };

    // let ranges = parse_pos(&fields)?;

    Ok(Config {
        files,
        delimiter: *delim_bytes.first().unwrap(),
        extract: ranges,
    })
}

fn parse_pos2(range: &str) -> MyResult<PositionList> {
    let mut range_vec = Vec::default();

    let extracts = range.split(",");

    for extract in extracts {
        if extract.contains("+") || extract.contains(char::is_alphabetic) {
            return Err(format!("illegal list value: \"{}\"", extract).into());
        }

        let split_extracts = extract.split("-").collect::<Vec<&str>>();
        if split_extracts.len() == 1 {
            match extract.parse::<usize>() {
                Ok(e) if e > 0 => range_vec.push(Range {
                    start: e - 1,
                    end: e,
                }),
                _ => {
                    return Err(format!("illegal list value: \"{}\"", range).into());
                }
            }
        } else if split_extracts.len() == 2 {
            let mut start = 0;
            let mut end = 0;

            for split_extract in split_extracts {
                match split_extract.parse::<usize>() {
                    Ok(e) if e > 0 => {
                        if start == 0 {
                            start = e
                        } else {
                            end = e
                        }
                    }
                    _ => {
                        return Err(format!("illegal list value: \"{}\"", split_extract).into());
                    }
                }
            }

            if start >= end {
                return Err(format!(
                    "First number in range ({}) must be lower than second number ({})",
                    start, end
                )
                .into());
            }

            range_vec.push(Range {
                start: start - 1,
                end,
            });
        } else {
            return Err(format!("illegal list value: \"{}\"", range).into());
        }
    }

    Ok(range_vec)
    // Vec<Range<usize>>
}

fn parse_index(input: &str) -> Result<usize, String> {
    let value_error = || format!("illegal list value: \"{}\"", input);
    input
        .starts_with("+")
        .then(|| Err(value_error()))
        .unwrap_or_else(|| {
            input
                .parse::<std::num::NonZeroUsize>()
                .map(|n| usize::from(n) - 1)
                .map_err(|_| value_error())
        })
}

fn parse_pos(range: &str) -> MyResult<PositionList> {
    let range_re = Regex::new(r"^(\d+)-(\d+)$").unwrap();
    range
        .split(',')
        .into_iter()
        .map(|pos_list| {
            parse_index(pos_list).map(|n| n..n + 1).or_else(|e| {
                range_re.captures(pos_list).ok_or(e).and_then(|captures| {
                    let n1 = parse_index(&captures[1])?;
                    let n2 = parse_index(&captures[2])?;
                    if n1 >= n2 {
                        return Err(format!(
                            "First number in range ({}) must be lower than second number ({})",
                            n1 + 1,
                            n2 + 1,
                        ));
                    }
                    Ok(n1..n2 + 1)
                })
            })
        })
        .collect::<Result<_, _>>()
        .map_err(From::from)
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}


fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    let chars: Vec<_> = line.chars().collect();
    let mut entries: Vec<char> = vec![];
    // for pos in char_pos.iter().cloned() {
    //     // for i in pos {
    //     //     if let Some(val) = chars.get(i) {
    //     //         entries.push(*val);
    //     //     }
    //     // }
    //     entries.extend(pos.filter_map(|i| chars.get(i)));
    // }
    // char_pos
    //     .iter()
    //     .cloned()
    //     .map(|range| range.filter_map(|i| chars.get(i)))
    //     .flatten()
    //     .collect()

    char_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| chars.get(i)))
        .collect()
}

// fn extract_fields(record: &StringRecord, field_pos: &[Range<usize>]) -> Vec<String> {
//     field_pos
//         .iter()
//         .cloned()
//         .flat_map(|range| range.filter_map(|i| record.get(i)))
//         .map(String::from)
//         .collect()
// }

fn extract_fields<'a>(record: &'a StringRecord, field_pos: &'a [Range<usize>]) -> Vec<&'a str> {
    field_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| record.get(i)))
        .collect()
}

fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    let bytes = line.as_bytes();
    let entries: Vec<_> = byte_pos
        .iter()
        .clone()
        .flat_map(|range| range.clone().filter_map(|i| bytes.get(i)).copied())
        .collect();
    String::from_utf8_lossy(&entries).into_owned()
}

#[cfg(test)]
mod unit_test {

    use csv::StringRecord;

    use super::{extract_bytes, extract_chars, extract_fields, parse_pos};

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 1..2, 4..5]), "áb".to_string());
    }

    #[test]
    fn test_extract_fields() {
        let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
        assert_eq!(extract_fields(&rec, &[0..1, 2..3]), &["Captain", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"]);
    }

    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }

    #[test]
    fn test_parse_pos() {
        // SAD PATH TESTS

        // empty string is an error
        assert!(parse_pos("").is_err());

        // zero is an error
        let res = parse_pos("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"");

        let res = parse_pos("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"");

        // leading "+" is an error
        let res = parse_pos("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1\"");

        let res = parse_pos("+1-2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1-2\"");

        let res = parse_pos("1-+2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-+2\"");

        // non-number is an error
        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"");

        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"");

        let res = parse_pos("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-a\"");

        let res = parse_pos("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a-1\"");

        // broken ranges
        let res = parse_pos("-");
        assert!(res.is_err());

        let res = parse_pos(",");
        assert!(res.is_err());

        let res = parse_pos("1,");
        assert!(res.is_err());

        let res = parse_pos("1-");
        assert!(res.is_err());

        let res = parse_pos("1-1-1");
        assert!(res.is_err());

        let res = parse_pos("1-1-a");
        assert!(res.is_err());

        // first number must be less than second
        let res = parse_pos("1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );

        let res = parse_pos("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );

        // HUZZAH! PATH TESTS
        let res = parse_pos("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);

        let res = parse_pos("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => {
                match &config.extract {
                    Extract::Fields(field_pos) => {
                        let mut reader = ReaderBuilder::new()
                            .delimiter(config.delimiter)
                            .has_headers(false)
                            .from_reader(file);
                        let mut wtr = WriterBuilder::new()
                            .delimiter(config.delimiter)
                            .from_writer(std::io::stdout());
                        for record in reader.records() {
                            wtr.write_record(extract_fields(&record?, field_pos))?;
                        }
                    }
                    Extract::Bytes(byte_pos) => {
                        for line in file.lines() {
                            println!("{}", extract_bytes(&line?, byte_pos));
                        }
                    }
                    Extract::Chars(char_pos) => {
                        for line in file.lines() {
                            println!("{}", extract_chars(&line?, char_pos));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
