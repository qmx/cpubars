use std;

#[derive(Debug, PartialEq, Eq)]
pub struct Stat {
    pub cores: Vec<CoreInfo>,
}

impl Stat {
    pub fn new(mut cores: Vec<CoreInfo>) -> Stat {
        cores.sort();
        Stat { cores }
    }
}

#[derive(Debug, Default)]
pub struct CoreInfo {
    pub id: u32,
    pub user: u32,
    pub nice: u32,
    pub system: u32,
    pub idle: u32,
    pub iowait: u32,
    pub irq: u32,
    pub softirq: u32,
    pub steal: u32,
    pub guest: u32,
    pub guest_nice: u32,
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

impl CpuUtilization {
    fn show_bars(&self) -> String {
        let bars: Vec<char> = "_▁▂▃▄▅▆▇█".chars().collect();
        self.cores
            .iter()
            .map(|c| c.utilization)
            .map(|v| (v * 100.0) as usize)
            .map(|v| v / 12)
            .map(|i| bars[i])
            .collect::<String>()
    }
}

impl std::fmt::Display for CpuUtilization {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.show_bars())
    }
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

#[cfg(test)]
mod test {
    use super::CoreInfo;
    use parser;

    #[test]
    fn test_stat_sub() {
        let d1 = include_bytes!("../fixtures/stress1_16cpu.0");
        let d2 = include_bytes!("../fixtures/stress1_16cpu.1");
        let s1 = parser::parse(d1.to_vec()).unwrap();
        let s2 = parser::parse(d2.to_vec()).unwrap();
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
