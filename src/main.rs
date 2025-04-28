use clap::Parser;
use anyhow::Result;
use nix::unistd::Pid;

use crate::linux::linux_seer;
use crate::windows::windows_seer;
pub mod linux;
pub mod windows;

#[derive(Parser, Debug)]
#[command(version, about, long_about = "Dump memory of a process.")]
struct Args {
    #[arg(short, long, help = "PID of target process.")]
    pid: i32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let pid = Pid::from_raw(args.pid);

    env_logger::init();

    if cfg!(target_os = "linux") {
        let mem = linux_seer::Mem::new(pid);
        mem?.dump()?;
    } else if cfg!(target_os = "windows") {
        windows_seer::dump(pid)?;
    }

    Ok(())
}
