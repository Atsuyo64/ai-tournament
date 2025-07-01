use crate::server::*;

//TODO: rename module

#[derive(Debug)]
pub struct AvailableRessources {
    pub available_cpus: i32,
    pub available_megabytes: i32,
}

impl From<crate::server::SystemParams> for AvailableRessources {
    fn from(value: crate::server::SystemParams) -> Self {
        let mut sys = sysinfo::System::new();

        let available_cpus = match value.cpus() {
            AvailableCPUs::Auto => {
                sys.refresh_cpu_all();
                sys.cpus().len() as i32
            }
            AvailableCPUs::Limited(limit) => *limit as i32,
        };

        let available_megabytes = match value.max_memory() {
            MaxMemory::Auto => {
                sys.refresh_memory();
                //Auto => use all memory except for 1GB //REVIEW: use 90% ?
                (sys.available_memory() / 1_000_000) as i32 - 1_000
            }
            MaxMemory::MaxMegaBytes(max) => *max as i32,
            MaxMemory::MaxGigaBytes(max) => (*max * 1_000) as i32,
        };

        assert!(available_cpus > 0, "Not enough CPUs to process");
        assert!(available_megabytes > 0, "Not enough memory to process");

        AvailableRessources {
            available_cpus,
            available_megabytes,
        }
    }
}

//TODO: implement
#[derive(Debug)]
pub struct GlobalResourceLimit {
    megabytes: u32,
    cpus: std::collections::HashSet<u8>, //Vec ?
    megabytes_per_agent: u32,
    cpu_per_agent: u8,
}

#[derive(Debug)]
pub struct MatchResourceLimit {
    megabytes_per_agent: u32,
    cpus: Vec<u8>,
    cpu_per_agent: u8,
}

//FIXME: temporary
impl MatchResourceLimit {
    pub fn empty() -> MatchResourceLimit {
        MatchResourceLimit { megabytes_per_agent: 0, cpus: vec![], cpu_per_agent: 0 }
    }
}
