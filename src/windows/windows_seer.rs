// dump memory of windows process
use nix::unistd::Pid;
use anyhow::Result;

pub fn dump(pid: Pid) -> Result<()> {
    println!("windows dumping {}", pid);
    Ok(())
}
