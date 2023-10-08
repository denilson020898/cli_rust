use clap::{App, Arg};
use std::error::Error;

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
                .value_name("COUNT")
                .short("c")
                .long("count")
                .help("Show counts"),
        )
        .get_matches();

    // let in_file = matches.value_of_lossy("infile").unwrap().to_string();
    let in_file = matches.value_of_lossy("infile").map(String::from).unwrap();
    let count = matches.is_present("count");
    // let out_file = matches
    //     .value_of_lossy("outfile")
    //     .and_then(|sq| Some(sq.to_string()));
    let out_file = matches.value_of("out_file").map(String::from);

    Ok(Config {
        in_file,
        out_file,
        count,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    dbg!(config);
    Ok(())
}
