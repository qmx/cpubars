const BARS: &'static str = "▁ ▂ ▃ ▄ ▅ ▆ ▇ █";

use std::path::Path;
use std::fs::File;
use std::io::Read;

fn main() {
    let path = Path::new("/proc/stat");

    let mut f = File::open(&path).unwrap();
    let mut s = Vec::new();
    f.read_to_end(&mut s).unwrap();
    println!("{:?}", parser::cpu_info(&s[..]));
    println!("{}", BARS);
}

#[macro_use]
extern crate nom;

mod parser {
    use nom::*;
    use std;

    #[derive(Debug, PartialEq, Default)]
    pub struct CpuInfo {
        id: Option<u32>,
        user: u32,
        nice: u32,
        system: u32,
        idle: u32,
        iowait: u32,
        irq: u32,
        softirq: u32,
        steal: u32,
        guest: u32,
        guest_nice: u32,
    }

    named!(
        counter<u32>,
        map_res!(
            map_res!(digit, std::str::from_utf8),
            std::str::FromStr::from_str
        )
    );

    named!(
        pub cpu<Vec<CpuInfo>>,
        fold_many1!(cpu_info, Vec::new(), |mut acc: Vec<_>, item| {
            acc.push(item);
            acc
        })
        );

    named!(
        pub cpu_info<CpuInfo>,
        do_parse!(
            tag!("cpu") >>
            id: opt!(counter) >>
            user: ws!(counter) >>
            nice: ws!(counter) >>
            system: ws!(counter) >>
            idle: ws!(counter) >>
            iowait: ws!(counter) >>
            irq: ws!(counter) >>
            softirq: ws!(counter) >>
            steal: ws!(counter) >>
            guest: ws!(counter) >>
            guest_nice: ws!(counter) >>
            (
                CpuInfo {
                    id: id,
                    user: user,
                    nice: nice,
                    system: system,
                    idle: idle,
                    iowait: iowait,
                    irq: irq,
                    softirq: softirq,
                    steal: steal,
                    guest: guest,
                    guest_nice: guest_nice,
                }
            )
        )
    );

    #[cfg(test)]
    mod test {
        use super::{cpu_info, CpuInfo};
        use nom::IResult;
        #[test]
        fn test_parse_aggregate_cpu() {
            let x = b"cpu  7378560 1341 419330 1234035738 849479 0 23487 0 0 0";
            match cpu_info(&x[..]) {
                IResult::Done(_, o) => {
                    assert_eq!(
                        o,
                        CpuInfo {
                            id: None,
                            user: 7378560,
                            nice: 1341,
                            system: 419330,
                            idle: 1234035738,
                            iowait: 849479,
                            irq: 0,
                            softirq: 23487,
                            steal: 0,
                            guest: 0,
                            guest_nice: 0,
                        }
                    );
                }
                _ => unreachable!(),
            }
        }

        #[test]
        fn test_parse_individual_cpu() {
            let x = b"cpu14  7378560 1341 419330 1234035738 849479 0 23487 0 0 0";
            match cpu_info(&x[..]) {
                IResult::Done(_, o) => {
                    assert_eq!(
                        o,
                        CpuInfo {
                            id: Some(14),
                            user: 7378560,
                            nice: 1341,
                            system: 419330,
                            idle: 1234035738,
                            iowait: 849479,
                            irq: 0,
                            softirq: 23487,
                            steal: 0,
                            guest: 0,
                            guest_nice: 0,
                        }
                    );
                }
                _ => unreachable!(),
            }
        }

        #[test]
        fn test_full_proc_stat() {
            let data = include_bytes!("../fixtures/sample_16cpu.0");
            match super::cpu(&data[..]) {
                IResult::Done(_, o) => {
                    assert_eq!(o.len(), 17);
                }
                _ => unreachable!(),
            }
        }
    }
}
