use clap::{App, Arg};
use std::{
    error::Error,
    fs::File,
    io::{stdin, stdout, BufRead, BufReader, Write},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    in_file: String,
    out_file: Option<String>,
    count: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("uniqr")
        .version("0.1.0")
        .author("Denilson Bro")
        .about("Rust uniq")
        .arg(
            Arg::with_name("infile")
                .value_name("IN_FILE")
                .help("Input file")
                .default_value("-"),
        )
        .arg(
            Arg::with_name("outfile")
                .value_name("OUT_FILE")
                .help("Output file"),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .help("Show counts")
                .long("count")
                .takes_value(false),
        )
        .get_matches();

    // let in_file = matches.value_of_lossy("infile").unwrap().to_string();
    let in_file = matches.value_of_lossy("infile").map(String::from).unwrap();
    let count = matches.is_present("count");
    // let out_file = matches
    //     .value_of_lossy("outfile")
    //     .and_then(|sq| Some(sq.to_string()));
    let out_file = matches.value_of("outfile").map(String::from);

    Ok(Config {
        in_file,
        out_file,
        count,
    })
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

pub fn run(config: Config) -> MyResult<()> {
    let mut file = open(&config.in_file).map_err(|e| format!("{}: {}", config.in_file, e))?;
    let mut out_file: Box<dyn Write> = match &config.out_file {
        Some(out_name) => Box::new(File::create(out_name)?),
        _ => Box::new(stdout()),
    };

    let mut line = String::new();
    let mut prev_line = String::new();

    let mut count = 0;

    let mut print = |count: u64, text: &str| -> MyResult<()> {
        if count > 0 {
            if config.count {
                write!(out_file, "{:>4} {}", count, text)?;
            } else {
                write!(out_file, "{}", text)?;
            }
        };
        Ok(())
    };

    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }

        if line.trim_end() != prev_line.trim_end() {
            print(count, &prev_line)?;
            prev_line = line.clone();
            count = 0;
        }

        count += 1;
        line.clear();
    }
    print(count, &prev_line)?;

    Ok(())
}
