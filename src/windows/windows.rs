// dump memory of windows process
use nix::unistd::Pid;
use anyhow::Result;

pub fn dump(pid: i32) -> Result<()> {
    let pid = Pid::from_raw(pid);
    println!("windows dumping {}", pid);
    Ok(())
}
