// code to dump live memory of a process into an ELF core file
use nix::{sys::ptrace, unistd::Pid};
use anyhow::Result;

pub fn dump(pid: Pid) -> Result<()> {
    println!("linux dumping {}", pid);
    ptrace::attach(pid)?;

    ptrace::detach(pid, None)?;
    Ok(())
}
