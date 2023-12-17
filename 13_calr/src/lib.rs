use std::{error::Error, str::FromStr};

use chrono::{Datelike, Local, NaiveDate};
use clap::{App, Arg};

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const LINE_WIDTH: usize = 22;

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
                .help("Year (1-9999)"),
        )
        .arg(
            Arg::with_name("month")
                .value_name("MONTH")
                .short("m")
                .help("Month name or number (1-12)"),
        )
        .arg(
            Arg::with_name("show_year")
                .short("y")
                .long("year")
                .help("Show whole current year")
                // .conflicts_with("month")
                .conflicts_with_all(&["month", "year"]),
        )
        .get_matches();

    let mut month = matches.value_of("month").map(parse_month).transpose()?;
    let mut year = matches.value_of("year").map(parse_year).transpose()?;

    let now = Local::now();

    if matches.is_present("show_year") {
        month = None;
        year = Some(now.year());
    } else if month.is_none() && year.is_none() {
        month = Some(now.month());
        year = Some(now.year());
    }

    Ok(Config {
        month,
        year: year.unwrap_or(now.year()),
        today: now.naive_local().into(),
    })
}

fn parse_int<T: FromStr>(val: &str) -> MyResult<T> {
    val.parse::<T>()
        .map_err(|_| format!("Invalid integer \"{}\"", val).into())
}

fn parse_year(year: &str) -> MyResult<i32> {
    // let year = parse_int::<i32>(year)?;
    // if year < 1 || year > 9999 {
    //     return Err(format!("year \"{}\" not in the range 1 through 9999", year).into());
    // }
    // Ok(year)

    parse_int(year).and_then(|num| {
        if (1..=9999).contains(&num) {
            Ok(num)
        } else {
            Err(format!("year \"{}\" not in the range 1 through 9999", year).into())
        }
    })
}

fn parse_month(month: &str) -> MyResult<u32> {
    // let month = parse_int::<u32>(month).map_err(|_| format!("Invalid month \"{}\"", month))?;
    // if month < 1 || month > 12 {
    //     return Err(format!("month \"{}\" not in the range 1 through 12", month).into());
    // }
    // Ok(month)

    match parse_int(month) {
        Ok(num) => {
            if (1..=12).contains(&num) {
                Ok(num)
            } else {
                Err(format!("month \"{}\" not in the range 1 through 12", month).into())
            }
        }
        _ => {
            let lower = &month.to_lowercase();
            let matches = MONTH_NAMES
                .iter()
                .enumerate()
                .filter_map(|(i, name)| {
                    if name.to_lowercase().starts_with(lower) {
                        Some(i + 1)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if matches.len() == 1 {
                Ok(matches[0] as u32)
            } else {
                Err(format!("Invalid month \"{}\"", month).into())
            }
        }
    }
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{parse_int, parse_month, parse_year};

    #[test]
    fn test_parse_int_as_usize() {
        let res = parse_int::<usize>("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1_usize);
    }

    #[test]
    fn test_parse_int_negative_as_i32() {
        let res = parse_int::<i32>("-1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), -1_i32);
    }

    #[test]
    fn test_parse_int_fail_on_string() {
        let res = parse_int::<i64>("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid integer \"foo\"");
    }

    #[test]
    fn test_parse_year_1_ok() {
        let res = parse_year("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1_i32);
    }

    #[test]
    fn test_parse_year_9999_ok() {
        let res = parse_year("9999");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 9999_i32);
    }

    #[test]
    fn test_parse_year_0_err() {
        let res = parse_year("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"0\" not in the range 1 through 9999"
        );
    }

    #[test]
    fn test_parse_year_10000_err() {
        let res = parse_year("10000");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"10000\" not in the range 1 through 9999"
        );
    }

    #[test]
    fn test_parse_year_foo_err() {
        let res = parse_year("foo");
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_month_1_ok() {
        let res = parse_month("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1_u32)
    }

    #[test]
    fn test_parse_month_12_ok() {
        let res = parse_month("12");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12_u32)
    }

    #[test]
    fn test_parse_month_jan_ok() {
        let res = parse_month("jan");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1_u32)
    }

    #[test]
    fn test_parse_month_0_err() {
        let res = parse_month("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"0\" not in the range 1 through 12"
        )
    }

    #[test]
    fn test_parse_month_13_err() {
        let res = parse_month("13");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"13\" not in the range 1 through 12"
        )
    }

    #[test]
    fn test_parse_month_foo_err() {
        let res = parse_month("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid month \"foo\"")
    }

    #[test]
    fn test_last_day_in_month() {
        assert_eq!(
            last_day_in_month(2019, 11),
            NaiveDate::from_ymd_opt(2019, 11, 30).unwrap()
        );
        assert_eq!(
            last_day_in_month(2023, 12),
            NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 1),
            NaiveDate::from_ymd_opt(2020, 1, 31).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 2),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 4),
            NaiveDate::from_ymd_opt(2020, 4, 30).unwrap()
        );
    }

    #[test]
    fn test_format_month_2020_2_ok() {
        let today = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, true, today), leap_february);
    }

    #[test]
    fn test_format_month_2020_5_ok() {
        let today = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let may = vec![
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, 5, false, today), may);
    }

    #[test]
    fn test_format_month_2021_4_ok() {
        let today = NaiveDate::from_ymd_opt(2021, 4, 7).unwrap();
        let april_with_highlight = vec![
            "      April 2021      ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        assert_eq!(format_month(2021, 4, true, today), april_with_highlight);
    }
}
