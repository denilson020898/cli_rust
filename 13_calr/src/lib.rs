use std::{error::Error, str::FromStr};

use chrono::{Datelike, Local, NaiveDate};
use clap::{App, Arg};
use itertools::izip;

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

fn format_month(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
    let mut result = vec![];

    let month_name = MONTH_NAMES.get(month as usize - 1).unwrap();
    let header = if print_year {
        format!("{} {}  ", month_name, year)
    } else {
        format!("{}  ", month_name)
    };

    result.push(format!("{:^width$}", header, width = LINE_WIDTH));
    result.push("Su Mo Tu We Th Fr Sa  ".to_string());

    let ymd_at_one = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let weekday = ymd_at_one.and_hms_opt(0, 0, 1).unwrap().weekday();
    let days_to_skip = match weekday {
        chrono::Weekday::Sun => 0_u32,
        _ => weekday.number_from_monday(),
    };

    let last_day = last_day_in_month(year, month);
    let last_day_num = last_day.day0();

    let mut counter_start = 0_usize;

    let days: Vec<_> = vec![Some(0); 7 * 6]
        .into_iter()
        .enumerate()
        .map(|(i, _e)| {
            if i < days_to_skip as usize || i > (last_day_num + days_to_skip) as usize {
                None
            } else {
                counter_start += 1;
                Some(counter_start)
            }
        })
        .collect();

    for week in days.chunks(7) {
        let week_formatted: Vec<_> = week
            .into_iter()
            .map(|w| match *w {
                Some(e) => {
                    let mut date_aligned = format!("{:>2}", e);
                    if year == today.year() && month == today.month() && today.day() as usize == e {
                        let reverse = ansi_term::Style::new().reverse();
                        date_aligned = format!("{}", reverse.paint(date_aligned));
                    }
                    date_aligned
                }
                None => "  ".to_string(),
            })
            .collect();
        let joined = week_formatted.join(" ");
        result.push(format!("{}  ", joined));
    }
    result
}

/// no need to handle panic here, too complicated
/// basically if date = 12, we can't go to next year (12 + 1) - 1 day by pred method
fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
    let next_month_first_day = NaiveDate::from_ymd_opt(year, month + 1, 1)
        .unwrap_or(NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap());
    next_month_first_day.pred_opt().unwrap()
}

pub fn run(config: Config) -> MyResult<()> {
    if let Some(month) = config.month {
        for line in format_month(config.year, month, true, config.today).iter() {
            println!("{}", line);
        }
    } else {
        println!("{:>32}", config.year);
        let months: Vec<_> = (1..13)
            .into_iter()
            .map(|month| format_month(config.year, month, false, config.today))
            .collect();
        for (i, chunk) in months.chunks(3).enumerate() {
            if let [m1, m2, m3] = chunk {
                for (w1, w2, w3) in izip!(m1, m2, m3) {
                    println!("{}{}{}", w1, w2, w3);
                }
                if i < 3 {
                    println!();
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::{format_month, last_day_in_month, parse_int, parse_month, parse_year};

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
            "     April 2021       ",
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

    #[test]
    fn test_format_month_2023_10_ok() {
        let today = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let october = vec![
            "    October 2023      ",
            "Su Mo Tu We Th Fr Sa  ",
            " 1  2  3  4  5  6  7  ",
            " 8  9 10 11 12 13 14  ",
            "15 16 17 18 19 20 21  ",
            "22 23 24 25 26 27 28  ",
            "29 30 31              ",
            "                      ",
        ];
        assert_eq!(format_month(2023, 10, true, today), october);
    }
}
