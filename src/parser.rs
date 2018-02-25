use nom::*;
use failure;
use std;

use model::{CoreInfo, Stat};

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

pub fn parse<'a>(d: Vec<u8>) -> Result<Stat, failure::Error> {
    match stat(&d[..]) {
        IResult::Done(_, stat) => Ok(stat),
        _ => Err(failure::err_msg("parse failure")),
    }
}
#[cfg(test)]
mod test {
    use super::{aggregate_cpu_info, cpu_info};
    use model::CoreInfo;
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

}
