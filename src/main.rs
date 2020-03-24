use std::{thread, time};

use psutil::cpu::cpu_times_percpu;

use clap::{crate_authors, crate_description, crate_name, crate_version, value_t, App, Arg};

fn main() -> anyhow::Result<()> {
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

    let bars: Vec<char> = " ▁▂▃▄▅▆▇█".chars().collect();

    let t1 = cpu_times_percpu()?;
    thread::sleep(time::Duration::from_millis(delay));
    let t2 = cpu_times_percpu()?;
    let result: String = t2
        .iter()
        .zip(t1.iter())
        .map(|(a, b)| a - b)
        .map(|c| (c.total().as_secs_f64() - c.idle().as_secs_f64()) / c.total().as_secs_f64())
        .map(|v| (v * 100.0) as usize)
        .map(|v| v / 12)
        .map(|i| bars[i])
        .collect();

    println!("{}", result);
    Ok(())
}
