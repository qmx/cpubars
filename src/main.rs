mod model;
mod parser;

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{thread, time};

use crate::model::Stat;
use anyhow;

use clap::{crate_authors, crate_description, crate_name, crate_version, value_t, App, Arg};

fn main() {
    let m = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("delay")
                .short("d")
                .long("delay")
                .value_name("DELAY")
                .help("delay in miliseconds")
                .default_value("100"),
        )
        .get_matches();

    let delay = value_t!(m, "delay", u64).unwrap_or(100);

    let s1 = get_stat().unwrap();
    let d = time::Duration::from_millis(delay);
    thread::sleep(d);
    let s2 = get_stat().unwrap();

    let utilization = s2 - s1;
    println!("{}", utilization);
}

fn get_stat<'a>() -> Result<Stat, anyhow::Error> {
    let path = Path::new("/proc/stat");
    let mut f = File::open(&path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    parser::parse(&s)
}
