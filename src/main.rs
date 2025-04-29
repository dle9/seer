use clap::Parser;
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(version, about, long_about = "Dump memory of a process.")]
struct Args {
    #[arg(short, long, help = "PID of target process.")]
    pid: i32,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    if cfg!(target_os = "linux") {
        let mut mem = seer::Mem::new()?;
        mem.set_pid(args.pid);
        mem.dump()?;
    } else if cfg!(target_os = "windows") {
        windows_seer::dump(args.pid)?;
    }

    Ok(())
}

use crate::linux::seer;
use crate::windows::windows_seer;

mod linux;
mod windows;
