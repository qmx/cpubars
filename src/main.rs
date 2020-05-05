use std::{thread, time};

use clap::{crate_authors, crate_description, crate_name, crate_version, value_t, App, Arg};
use crossterm::{
    execute,
    style::{Color as TermColor, Print, SetForegroundColor},
    Command,
};
use psutil::cpu::cpu_times_percpu;
use std::{
    fmt,
    io::{stdout, Write},
};

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
        .arg(
            Arg::with_name("color")
                .short("c")
                .long("color")
                .multiple(false)
                .help("Color the bars green/yellow/red depending on load. Default is no color"),
        )
        .arg(
            Arg::with_name("tmux-color")
                .short("C")
                .long("tmux-color")
                .multiple(false)
                .help("Same as -c/--color but will use tmux color codes rather than ansi codes"),
        )
        .get_matches();

    let delay = value_t!(m, "delay", u64).unwrap_or(100);

    let bars: Vec<char> = " ▁▂▃▄▅▆▇█".chars().collect();

    let t1 = cpu_times_percpu()?;
    thread::sleep(time::Duration::from_millis(delay));
    let t2 = cpu_times_percpu()?;
    let (ratios, result) = t2
        .iter()
        .zip(t1.iter())
        .map(|(a, b)| a - b)
        .map(|c| (c.total().as_secs_f64() - c.idle().as_secs_f64()) / c.total().as_secs_f64())
        .map(|v| (v, (v * 100.0) as usize))
        .map(|(r, v)| (r, v / 12))
        .map(|(r, i)| (r, bars[i]))
        .unzip::<_, _, Vec<_>, String>();

    let include_color = m.occurrences_of("color") == 1;
    let use_tmux_color = m.occurrences_of("tmux-color") == 1;

    if include_color && use_tmux_color {
        eprintln!("You cannot set both -c/--color and -C/--tmux-color");
        std::process::exit(1);
    }

    if include_color {
        print_with_colors(result, ratios, ColorMode::Ansi);
    } else if use_tmux_color {
        print_with_colors(result, ratios, ColorMode::Tmux);
    } else {
        print!("{}", result);
    }

    Ok(())
}

const GREEN_UNTIL: f64 = 2.0;
const YELLOW_UNTIL: f64 = 8.0;

fn print_with_colors(result: String, ratios: Vec<f64>, color_mode: ColorMode) {
    let color = get_color(ratios);
    let color_start = color.command(&color_mode);
    let color_end = color_mode.reset();

    execute!(stdout(), color_start, Print(result), color_end).unwrap();
}

fn get_color(ratios: Vec<f64>) -> Color {
    let sum = ratios.iter().sum::<f64>();

    if 0.0 <= sum && sum < GREEN_UNTIL {
        Color::Green
    } else if GREEN_UNTIL <= sum && sum < YELLOW_UNTIL {
        Color::Yellow
    } else {
        Color::Red
    }
}

#[derive(Debug)]
enum Color {
    Green,
    Yellow,
    Red,
}

impl Color {
    fn command(&self, color_mode: &ColorMode) -> impl Command {
        match color_mode {
            ColorMode::Ansi => Either::A(self.ansi_command()),
            ColorMode::Tmux => Either::B(self.tmux_command()),
        }
    }

    fn ansi_command(&self) -> impl Command {
        let color = match self {
            Color::Green => TermColor::Green,
            Color::Yellow => TermColor::Yellow,
            Color::Red => TermColor::Red,
        };
        SetForegroundColor(color)
    }

    fn tmux_command(&self) -> impl Command {
        let name = match self {
            Color::Green => "green",
            Color::Yellow => "yellow",
            Color::Red => "red",
        };
        Print(format!("#[fg={}]", name))
    }
}

#[derive(Debug)]
enum ColorMode {
    Ansi,
    Tmux,
}

impl ColorMode {
    fn reset(&self) -> impl Command {
        match self {
            ColorMode::Ansi => Either::A(SetForegroundColor(TermColor::Reset)),
            ColorMode::Tmux => Either::B(Print("#[fg=default]")),
        }
    }
}

#[derive(Debug)]
enum Either<A, B> {
    A(A),
    B(B),
}

impl<A: fmt::Display, B: fmt::Display> fmt::Display for Either<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Either::A(a) => write!(f, "{}", a),
            Either::B(b) => write!(f, "{}", b),
        }
    }
}

impl<A: Command, B: Command> Command for Either<A, B> {
    type AnsiType = Either<A::AnsiType, B::AnsiType>;

    fn ansi_code(&self) -> Self::AnsiType {
        match self {
            Either::A(a) => Either::A(a.ansi_code()),
            Either::B(b) => Either::B(b.ansi_code()),
        }
    }
}
