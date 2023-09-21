use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("catr")
        .version("0.1.0")
        .author("Denilson <denilson020898@gmail.com>")
        .about("Cat written in Rust")
        .arg(
            Arg::with_name("files")
                .value_name("FILES")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("number")
                .short("n")
                .long("number")
                .help("Number lines")
                .takes_value(false)
                .conflicts_with("number_nonblank"),
        )
        .arg(
            Arg::with_name("number_nonblank")
                .short("b")
                .long("number-nonblank")
                .help("Number non-blank lines")
                .takes_value(false),
        )
        .get_matches();

    let files = matches.values_of_lossy("files").unwrap();
    let number_lines = matches.is_present("number");
    let number_nonblank_lines = matches.is_present("number_nonblank");

    Ok(Config {
        files,
        number_lines,
        number_nonblank_lines,
    })
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

// will not compile
// fn open2(filename: &str) -> MyResult<dyn BufRead> {
//     match filename {
//         "-" => Ok(BufReader::new(io::stdin())),
//         _ => Ok(BufReader::new(File::open(filename)?)),
//     }
// }

pub fn run(config: Config) -> MyResult<()> {
    for filename in config.files {
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {}: {}", filename, err),
            Ok(file) => {
                let mut line_num = 0;
                for line in file.lines() {
                    let line = line?;

                    line_num += 1;

                    if config.number_lines {
                        println!("{:>6}\t{}", line_num, line);
                    } else if config.number_nonblank_lines {
                        if line.is_empty() {
                            line_num -= 1;
                            println!();
                        } else {
                            println!("{:>6}\t{}", line_num, line);
                        }
                    } else {
                        println!("{}", line);
                    }
                }
            }
        }
    }

    Ok(())
}
