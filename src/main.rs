const BARS: &'static str = "▁ ▂ ▃ ▄ ▅ ▆ ▇ █";

use std::path::Path;
use std::fs::File;
use std::io::Read;
use nom::IResult::Done;

fn main() {
    let path = Path::new("/proc/stat");

    let mut f = File::open(&path).unwrap();
    let mut s = Vec::new();
    f.read_to_end(&mut s).unwrap();
    if let Done(_, stat) = parser::stat(&s[..]) {
        println!("{:?}", stat);
    }

    println!("{}", BARS);
}

#[macro_use]
extern crate nom;

mod parser {
    use nom::*;
    use std;

    #[derive(Debug, PartialEq, Eq)]
    pub struct Stat {
        cores: Vec<CoreInfo>,
    }

    #[derive(Debug)]
    pub struct CoreInfo {
        id: u32,
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

    impl Ord for CoreInfo {
        fn cmp(&self, other: &CoreInfo) -> std::cmp::Ordering {
            self.id.cmp(&other.id)
        }
    }

    impl PartialOrd for CoreInfo {
        fn partial_cmp(&self, other: &CoreInfo) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl PartialEq for CoreInfo {
        fn eq(&self, other: &CoreInfo) -> bool {
            self.id == other.id
        }
    }

    impl Eq for CoreInfo {}

    pub struct CpuUtilization {
        aggregate: f64,
    }

    impl std::ops::Sub for Stat {
        type Output = CpuUtilization;

        fn sub(self, other: Stat) -> CpuUtilization {
            CpuUtilization { aggregate: 0.0 }
        }
    }

    named!(
        counter<u32>,
        map_res!(
            map_res!(digit, std::str::from_utf8),
            std::str::FromStr::from_str
        )
    );

    named!(
        individual_cores<Vec<CoreInfo>>,
        fold_many1!(cpu_info, Vec::new(), |mut acc: Vec<_>, item| {
            acc.push(item);
            acc
        })
    );

    named!(
        pub stat<Stat>,
        do_parse!(
            aggregate: aggregate_cpu_info >>
            cores: individual_cores >>
            (Stat{cores:cores})
            )
        );

    named!(
        aggregate_cpu_info<()>,
        do_parse!(
            tag!("cpu ") >>
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
                ()
            )
        )
    );

    named!(
        cpu_info<CoreInfo>,
        do_parse!(
            tag!("cpu") >>
            id: counter >>
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
                CoreInfo {
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
        use super::{aggregate_cpu_info, cpu_info, CoreInfo};
        use nom::IResult;
        #[test]
        fn test_parse_aggregate_cpu() {
            let x = b"cpu  7378560 1341 419330 1234035738 849479 0 23487 0 0 0";
            match aggregate_cpu_info(&x[..]) {
                IResult::Done(_, o) => {
                    assert_eq!(o, ());
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
                        CoreInfo {
                            id: 14,
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
            match super::stat(&data[..]) {
                IResult::Done(_, o) => {
                    assert_eq!(o.cores.len(), 16);
                }
                _ => unreachable!(),
            }
        }

        #[test]
        fn test_stat_sub() {
            let d1 = include_bytes!("../fixtures/stress1_16cpu.0");
            let d2 = include_bytes!("../fixtures/stress1_16cpu.1");
            let s1 = parse(&d1[..]).unwrap();
            let s2 = parse(&d2[..]).unwrap();
            let utilization = s2 - s1;
        }

        fn parse<'a>(d: &'a [u8]) -> Result<super::Stat, &'a str> {
            match super::stat(d) {
                IResult::Done(_, stat) => Ok(stat),
                _ => Err("parse failure"),
            }
        }
    }
}
