
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::{thread, time};

fn main() {
    let bars: Vec<char> = "_▁▂▃▄▅▆▇█".chars().collect();
    let s1 = get_stat().unwrap();
    let d = time::Duration::from_millis(100);
    thread::sleep(d);
    let s2 = get_stat().unwrap();

    let utilization = s2 - s1;
    let values = utilization
        .cores
        .iter()
        .map(|c| c.utilization)
        .map(|v| (v * 100.0) as usize)
        .map(|v| v / 12)
        .map(|i| bars[i])
        .collect::<String>();
    println!("{}", values);
}

fn get_stat<'a>() -> Result<parser::Stat, &'a str> {
    let path = Path::new("/proc/stat");
    let mut f = File::open(&path).unwrap();
    let mut s = Vec::new();
    f.read_to_end(&mut s).unwrap();
    let stat = parser::parse(s);
    stat
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

    impl Stat {
        fn new(mut cores: Vec<CoreInfo>) -> Stat {
            cores.sort();
            Stat { cores }
        }
    }

    #[derive(Debug, Default)]
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

    impl CoreInfo {
        fn total(&self) -> f64 {
            (self.user + self.nice + self.system + self.idle + self.iowait + self.irq + self.softirq
                + self.steal) as f64
        }

        fn idle(&self) -> f64 {
            (self.idle + self.iowait) as f64
        }
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

    #[derive(Debug)]
    pub struct CpuUtilization {
        pub cores: Vec<CoreUtilization>,
    }

    #[derive(Debug)]
    pub struct CoreUtilization {
        id: u32,
        pub utilization: f64,
    }

    impl<'a> std::ops::Sub for &'a CoreInfo {
        type Output = CoreUtilization;
        fn sub(self, other: &'a CoreInfo) -> CoreUtilization {
            let total = self.total() - other.total();
            let idle = self.idle() - other.idle();
            CoreUtilization {
                id: self.id,
                utilization: (total - idle) / total,
            }
        }
    }

    impl std::ops::Sub for Stat {
        type Output = CpuUtilization;

        fn sub(self, other: Stat) -> CpuUtilization {
            let cores_utilization = self.cores
                .iter()
                .zip(other.cores.iter())
                .map(|(a, b)| a - b)
                .collect();

            CpuUtilization {
                cores: cores_utilization,
            }
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
            (Stat::new(cores))
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
    pub fn parse<'a>(d: Vec<u8>) -> Result<Stat, &'a str> {
        match stat(&d[..]) {
            IResult::Done(_, stat) => Ok(stat),
            _ => Err("parse failure"),
        }
    }

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
            let s1 = super::parse(d1.to_vec()).unwrap();
            let s2 = super::parse(d2.to_vec()).unwrap();
            let utilization = s2 - s1;
        }


        #[test]
        fn test_sorted_cores() {
            let c1 = CoreInfo {
                id: 3,
                ..Default::default()
            };
            let c2 = CoreInfo {
                id: 0,
                ..Default::default()
            };
            let mut v = vec![c1, c2];
            v.sort();
            assert_eq!(v[0].id, 0);
        }
    }
}
