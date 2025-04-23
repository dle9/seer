use log::{info, error};
use nix::{sys::{ptrace}, unistd::Pid};
use std::fs::read_to_string;
use anyhow::Result;


#[derive(Debug)]
/// Data structure for /proc/<pid>/maps
struct MapData {
    /// Memory address block in the format `<addr start>-<addr end>`.
    addr_block: Option<String>,

    /// Permissions: `r`, `w`, `x`, `p` (private), `s` (shared).
    perms: Option<String>,

    /// Offset within the file for file-backed mappings.
    offset: Option<String>,

    /// Device major and minor IDs in the format `<major>:<minor>`.
    device: Option<String>,

    /// Inode number of the file for file-backed mappings.
    inode: Option<String>,

    /// Filesystem path for file-backed mappings.
    pathname: Option<String>,
}

/// Dump memory of linux process
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

fn get_map_data(pid: Pid) -> Result<Vec<MapData>> {
    let raw_data = read_to_string(format!("/proc/{pid}/maps"))
        .expect("Failed to read mapping");

    let lines: Vec<Vec<&str>> = raw_data
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
