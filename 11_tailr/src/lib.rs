use clap::{App, Arg};
use num::Zero;
use once_cell::sync::OnceCell;
use regex::Regex;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::ops::Deref;
use std::{error::Error, fs::File};

// static NUM_RE: OnceCell<Regex> = OnceCell::new();

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq)]
enum TakeValue {
    PlusZero,
    TakeNum(i64),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: TakeValue,
    bytes: Option<TakeValue>,
    quiet: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("tailr")
        .version("0.1.0")
        .author("Denilson <denilson020898@gmail.com>")
        .about("Rust tail")
        // something
        .arg(
            Arg::with_name("file")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .required(true),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help("Suppress headers"),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .help("Number of bytes")
                .value_name("BYTES")
                .conflicts_with("lines"),
        )
        .arg(
            Arg::with_name("lines")
                .short("n")
                .long("lines")
                .help("Number of lines")
                .value_name("LINES")
                .default_value("10"),
        )
        .get_matches();

    let files = matches.values_of_lossy("file").unwrap();
    let quiet = matches.is_present("quiet");

    let bytes = matches
        .value_of("bytes")
        .map(parse_take_num)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;

    let lines = matches
        .value_of("lines")
        .map(parse_take_num)
        .transpose()
        .map_err(|e| format!("illegal line count -- {}", e))?
        .unwrap();

    Ok(Config {
        files,
        lines,
        bytes,
        quiet,
    })
}

fn parse_take_num(value: &str) -> MyResult<TakeValue> {
    match value.parse::<i64>() {
        Ok(_) if value == "+0" => Ok(TakeValue::PlusZero),
        Ok(n) => Ok(TakeValue::TakeNum(
            if value.starts_with("+") || value.starts_with("-") {
                1
            } else {
                -1
            } * n,
        )),
        _ => Err(value.into()),
    }
}

// fn parse_take_num(value: &str) -> MyResult<TakeValue> {
//     let num_re = NUM_RE.get_or_init(|| Regex::new(r"^([+-])?(\d+)$").unwrap());
//     match num_re.captures(value) {
//         Some(caps) => {
//             let sign = caps.get(1).map_or("-", |m| m.as_str());
//             let num = format!("{}{}", sign, caps.get(2).unwrap().as_str());
//             if let Ok(value) = num.parse() {
//                 if sign == "+" && value == 0 {
//                     Ok(TakeValue::PlusZero)
//                 } else {
//                     Ok(TakeValue::TakeNum(value))
//                 }
//             } else {
//                 Err(From::from(value))
//             }
//         }
//         _ => Err(From::from(value)),
//     }
// }

fn count_lines_bytes(filename: &str) -> MyResult<(i64, i64)> {
    let file = File::open(filename)?;
    let mut filehandle = BufReader::new(file);

    let mut num_lines = 0;
    let mut num_bytes = 0;

    let mut buf = String::new();
    while let Ok(read_bytes) = filehandle.read_line(&mut buf) {
        if read_bytes == 0 {
            break;
        }
        num_bytes += read_bytes as i64;
        num_lines += 1;
        buf.clear()
    }

    Ok((num_lines, num_bytes))
}

fn print_lines(mut file: impl BufRead, num_lines: &TakeValue, total_lines: i64) -> MyResult<()> {
    unimplemented!()
}

fn print_bytes<T: Read + Seek>(
    mut file: T,
    num_lines: &TakeValue,
    total_lines: i64,
) -> MyResult<()> {
    unimplemented!()
}

fn get_start_index(take_val: &TakeValue, total: i64) -> Option<u64> {
    if total == 0 {
        return None;
    }
    match take_val {
        TakeValue::PlusZero => {
            let start_index = total as u64 - 1;
            return Some(start_index);
        }
        TakeValue::TakeNum(ref num) => {
            if num.is_zero() || *num > total {
                return None;
            } else if num.is_negative() {

                if num.abs() > total {
                    return Some(0);
                }

                let start_index = total + num;
                return Some(start_index as u64);
            } else {
                let start_index = *num as u64 - 1;
                return Some(start_index);
            }
        }
    }
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in config.files {
        match File::open(&filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(_) => {
                let (total_lines, total_bytes) = count_lines_bytes(&filename)?;
                println!(
                    "{} has {} lines and {} bytes",
                    filename, total_lines, total_bytes
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::get_start_index;

    use super::{count_lines_bytes, parse_take_num, TakeValue};

    #[test]
    fn test_parse_num() {
        let res = parse_take_num("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeValue::TakeNum(-3));

        let res = parse_take_num("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeValue::TakeNum(3));

        let res = parse_take_num("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeValue::TakeNum(-3));

        let res = parse_take_num("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeValue::TakeNum(0));

        let res = parse_take_num("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeValue::PlusZero);

        let res = parse_take_num(&i64::MAX.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeValue::TakeNum(i64::MIN + 1));

        let res = parse_take_num(&(i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeValue::TakeNum(i64::MIN + 1));

        let res = parse_take_num(&(i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeValue::TakeNum(i64::MIN + 1));

        let res = parse_take_num(&format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeValue::TakeNum(i64::MAX));

        let res = parse_take_num(&i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeValue::TakeNum(i64::MIN));

        let res = parse_take_num("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "3.14");

        let res = parse_take_num("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo");
    }

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");

        assert!(res.is_ok());

        assert_eq!(res.unwrap(), (1, 24));

        let res = count_lines_bytes("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (10, 49));
    }

    #[test]
    fn test_get_start_index() {
        // +0 for empty file, returns None
        assert_eq!(get_start_index(&TakeValue::PlusZero, 0), None);

        // +0 for nonempty file, return index one less than the number of line
        assert_eq!(get_start_index(&TakeValue::PlusZero, 1), Some(0));

        // Taking 0 lines/bytes, return 0
        assert_eq!(get_start_index(&TakeValue::TakeNum(0), 1), None);

        // Taking any lines/bytes from enty file, returns None
        assert_eq!(get_start_index(&TakeValue::TakeNum(1), 0), None);

        // Taking more lines/bytes than available from file, returns None
        assert_eq!(get_start_index(&TakeValue::TakeNum(2), 1), None);

        // starting line/byte is less than totao lines/bytes
        assert_eq!(get_start_index(&TakeValue::TakeNum(1), 10), Some(0));
        assert_eq!(get_start_index(&TakeValue::TakeNum(2), 10), Some(1));
        assert_eq!(get_start_index(&TakeValue::TakeNum(3), 10), Some(2));
        assert_eq!(get_start_index(&TakeValue::TakeNum(4), 10), Some(3));

        // negative starting line/byte is less than totao lines/bytes
        assert_eq!(get_start_index(&TakeValue::TakeNum(-1), 10), Some(9));
        assert_eq!(get_start_index(&TakeValue::TakeNum(-2), 10), Some(8));
        assert_eq!(get_start_index(&TakeValue::TakeNum(-3), 10), Some(7));
        assert_eq!(get_start_index(&TakeValue::TakeNum(-4), 10), Some(6));

        // negative starting line is more than total
        // return 0 to print the entire file
        let result = get_start_index(&TakeValue::TakeNum(-20), 10);
        assert_eq!(result, Some(0));
    }
}
