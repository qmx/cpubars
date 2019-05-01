use failure;

use crate::model::{CoreInfo, Stat};

pub fn parse(data: &str) -> Result<Stat, failure::Error> {
    fn parse_cpu_line(s: &str) -> Result<CoreInfo, failure::Error> {
        let line = s
            .split(" ")
            .map(|t| t.parse::<u32>().expect("failed to parse counter"))
            .collect::<Vec<_>>();

        Ok(CoreInfo {
            id: line[0],
            user: line[1],
            nice: line[2],
            system: line[3],
            idle: line[4],
            iowait: line[5],
            irq: line[6],
            softirq: line[7],
            steal: line[8],
            guest: line[9],
            guest_nice: line[10],
        })
    }

    let cpu_data = data
        .split("\n")
        .filter(|s| s.starts_with("cpu"))
        .collect::<Vec<&str>>();

    let cpus: Result<Vec<CoreInfo>, failure::Error> = cpu_data
        .iter()
        .skip(1)
        .map(|s| {
            if s.starts_with("cpu") {
                s.split("cpu").collect::<Vec<_>>()[1]
            } else {
                s
            }
        })
        .map(parse_cpu_line)
        .collect();

    Ok(Stat { cores: cpus? })
}

#[cfg(test)]
mod test {

    #[test]
    fn test_naive_parser() {
        let d1 = include_str!("../fixtures/sample_16cpu.0");
        let d2 = include_str!("../fixtures/sample_16cpu.1");
        let s1 = super::parse(d1).unwrap();
        let s2 = super::parse(d2).unwrap();
        assert_eq!("                ", format!("{}", s2 - s1));
    }

}
