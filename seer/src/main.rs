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

    let mut mem = Mem::new()?;
    mem.set_pid(args.pid);
    mem.dump()?;

    Ok(())
}

#[cfg(target_os = "linux")]
use linux::Mem;
