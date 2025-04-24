use log::{info, error};
use nix::{sys::ptrace, unistd::Pid};
use std::fs::{File, read_to_string};
use std::io::{Read, Seek};
use anyhow::Result;


/// for /proc/<pid>/maps
struct MapData {
    /// start addr of the mapping address block.
    start:      Option<String>,

    /// end addr of the mapping address block.
    end:        Option<String>,

    /// permissions: `r`, `w`, `x`, `p` (private), `s` (shared).
    perms:      Option<String>,

    /// offset within the file for file mappings.
    offset:     Option<String>,

    /// device id `<major>:<minor>`.
    device:     Option<String>,

    /// inode number of the file for file  mappings.
    inode:      Option<String>,

    /// filesystem path for file mappings.
    pathname:   Option<String>,
}


/// for /prov/<pid>/mem
pub struct Mem {
    /// pid of target process
    pid:        Pid,

    /// vector holding the entire file mapping file of the target pid
    map_data:       Vec<MapData>
}


impl Mem {
    pub fn new(_pid: Pid) -> Self {
        Self {
            pid: _pid,
            map_data: Vec::new(),
        }
    }

    /// dump memory of linux process
    pub fn dump(&mut self) -> Result<()> {
        match ptrace::attach(self.pid) {
            Ok(()) => {
                info!("ptrace::attach({})", self.pid);
                self.get_map_data();
                // self.get_mem_data();
            },
            Err(e) => error!("ptrace::attach({})", e)
        };

        match ptrace::detach(self.pid, None) {
            Ok(()) => info!("ptrace::detach({})", self.pid),
            Err(e) => error!("ptrace::detach({})", e)
        };

        Ok(())
    }    

    /// parse data from /proc/<pid>/maps
    pub fn get_map_data(&mut self) {
        let raw = read_to_string(format!("/proc/{}/maps", self.pid))
            .expect("Failed to read mapping");

        let lines: Vec<Vec<&str>> = raw
            .lines()
            .map(|line| {
                line.split_whitespace()
                    .collect::<Vec<&str>>()
            })
            .collect();
        
        let mut map_data: Vec<MapData> = Vec::new();

        for line in lines {
            let addr_block = line.get(0).map(|s| s.to_string());
            let (start, end) = addr_block.as_ref().unwrap()
                .split_once("-")
                .expect("Failed to read address block");

            map_data.push(MapData { 
                start:      Some(start.to_string()),
                end:        Some(end.to_string()),
                perms:      line.get(1).map(|s| s.to_string()), 
                offset:     line.get(2).map(|s| s.to_string()), 
                device:     line.get(3).map(|s| s.to_string()), 
                inode:      line.get(4).map(|s| s.to_string()), 
                pathname:   line.get(5).map(|s| s.to_string()) 
            }); 
        }

        self.map_data = map_data;
    }

    /// parse data from /proc/<pid>/mem 
    fn get_mem_data(&self) -> Result<()> {
        let mut raw_mem = File::open(format!("/proc/{}/mem", self.pid))
            .expect("Failed to read mem");

        for mapping in &self.map_data {
        }

        Ok(())
    }
}

