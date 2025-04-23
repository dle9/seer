use log::{info, error};
use nix::{sys::{ptrace}, unistd::Pid};
use std::fs::read_to_string;
use anyhow::Result;


/// data structure for /proc/<pid>/maps
struct MapData {
    /// memory address block in the format `<addr start>-<addr end>`.
    addr_block: Option<String>,

    /// permissions: `r`, `w`, `x`, `p` (private), `s` (shared).
    perms: Option<String>,

    /// offset within the file for file mappings.
    offset: Option<String>,

    /// device id `<major>:<minor>`.
    device: Option<String>,

    /// inode number of the file for file  mappings.
    inode: Option<String>,

    /// filesystem path for file mappings.
    pathname: Option<String>,
}

/// dump memory of linux process
pub fn dump(pid: Pid) -> Result<()> {
    match ptrace::attach(pid) {
        Ok(()) => {
            info!("ptrace::attach({})", pid);
            get_map_data(pid)?;
        },
        Err(e) => error!("ptrace::attach({})", e)
    };

    match ptrace::detach(pid, None) {
        Ok(()) => info!("ptrace::detach({})", pid),
        Err(e) => error!("ptrace::detach({})", e)
    };

    Ok(())
}

/// parse data from /proc/<pid>/maps
fn get_map_data(pid: Pid) -> Result<Vec<MapData>> {
    let raw_map_data = read_to_string(format!("/proc/{pid}/maps"))
        .expect("Failed to read mapping");

    let lines: Vec<Vec<&str>> = raw_map_data
        .lines()
        .map(|line| {
            line.split_whitespace()
                .collect::<Vec<&str>>()
        })
        .collect();
    
    let mut map_data: Vec<MapData> = Vec::new();
    for line in lines {
        map_data.push(MapData { 
            addr_block: line.get(0).map(|s| s.to_string()), 
            perms:      line.get(1).map(|s| s.to_string()), 
            offset:     line.get(2).map(|s| s.to_string()), 
            device:     line.get(3).map(|s| s.to_string()), 
            inode:      line.get(4).map(|s| s.to_string()), 
            pathname:   line.get(5).map(|s| s.to_string()) 
        }); 
    }

    Ok(map_data)
}

// parse data from /proc/<pid>/mem 
fn get_mem_data(map_data: &[MapData], pid: Pid) -> Result<()> {
    let mem = read_to_string(format!("/proc/{pid}/mem"));

    Ok(())
}
