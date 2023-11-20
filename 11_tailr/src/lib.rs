use clap::{App, Arg};
use std::error::Error;

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
                .takes_value(true)
                .multiple(true)
                .required(true),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help("Suppress headers")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .help("Number of bytes")
                .takes_value(true)
                .conflicts_with("lines"),
        )
        .arg(
            Arg::with_name("lines")
                .short("n")
                .long("lines")
                .help("Number of lines")
                .default_value("10")
                .takes_value(true),
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
            if value.starts_with("+") || value.starts_with("-"){ 1 } else { -1 } * n,
        )),
        _ => Err(value.into()),
    }
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_take_num, TakeValue};

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
}
