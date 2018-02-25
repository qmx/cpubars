#[macro_use]
extern crate nom;

extern crate failure;

mod model;
mod parser;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::{thread, time};

use model::Stat;

fn main() {
    let s1 = get_stat().unwrap();
    let d = time::Duration::from_millis(100);
    thread::sleep(d);
    let s2 = get_stat().unwrap();

    let utilization = s2 - s1;
    println!("{}", utilization);
}

fn get_stat<'a>() -> Result<Stat, failure::Error> {
    let path = Path::new("/proc/stat");
    let mut f = File::open(&path)?;
    let mut s = Vec::new();
    f.read_to_end(&mut s)?;
    parser::parse(s)
}
