use std::error::Error;

use chrono::{NaiveDate, Local, Datelike};
use clap::{App, Arg};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    month: Option<u32>,
    year: i32,
    today: NaiveDate,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("calr")
        .version("0.1.0")
        .author("Denilson <denilson020898@gmail.com.")
        .about("Rust cal")
        // Args
        .arg(
            Arg::with_name("year")
                .value_name("YEAR")
                .help("Year (1-9999)")
        )
        .arg(
            Arg::with_name("month")
                .value_name("MONTH")
                .short("m")
                .help("Month name or number (1-12)")
        )
        .arg(
            Arg::with_name("show_year")
                .short("y")
                .long("year")
                .help("Show whole current year")
        )
        .get_matches();

    let now = Local::now();

    Ok(Config {
        month: Some(now.month()),
        year: now.year(),
        today: now.naive_local().into()
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);
    Ok(())
}
