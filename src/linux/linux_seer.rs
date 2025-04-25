use log::{info, error};
use nix::{sys::{ptrace, wait}, unistd::Pid};
use std::fs::{File, read_to_string};
use std::io::{Read, Seek, SeekFrom};
use std::mem::MaybeUninit;
use anyhow::Result;

/// for /proc/<pid>/maps
#[derive(Debug)]
struct MapData {
    /// start addr of the mapping address block.
    start:      u64,

    /// end addr of the mapping address block.
    end:        u64,

    /// permissions: `r`, `w`, `x`, `p` (private), `s` (shared).
    r:          bool,
    w:          bool,
    x:          bool,
    p:          bool,
    s:          bool,

    /// offset within the file for file mappings.
    offset:     String,

    /// device id `<major>:<minor>`.
    device:     String,

    /// inode number of the file for file mappings.
    inode:      String,

    /// filesystem path for file mappings.
    pathname:   Option<String>,
}


/// for /prov/<pid>/mem
pub struct Mem {
    /// pid of target process
    pid:        Pid,

    /// vector holding the entire file mapping file of the target pid
    mapping:   Vec<MapData>,

    /// file of the memory
    mem:        File
}

impl Mem {
    pub fn new(_pid: Pid) -> Result<Self> {
        Ok(Self {
            pid: _pid,
            mapping: Vec::new(),
            mem: File::open(format!("/proc/{}/mem", _pid))?,
        })
    }

    /// dump memory of linux process
    pub fn dump(&mut self) -> Result<()> {
        ptrace::attach(self.pid).expect("failed to attach to pid.");

        match wait::waitpid(self.pid, None) { 
            Ok(wait::WaitStatus::Stopped(_, _)) => {
                info!("ptrace::attach({})", self.pid);
                self.read_mapping();

                // self.get_mem_data(addr)
                let first = self.mapping.first().unwrap();

                dbg!(self.read_mem_slice::<usize>(first.start, (first.end - first.start) as usize))?;

                ptrace::detach(self.pid, None).expect("failed to detach pid.");
            },

            Err(e) => error!("waitpid error {}", e),

            _ => ()

        };

        Ok(())
    }    

    /// parse data from /proc/<pid>/maps
    pub fn read_mapping(&mut self) {
        let raw = read_to_string(format!("/proc/{}/maps", self.pid))
            .expect("Failed to read mapping");

        let lines: Vec<Vec<&str>> = raw
            .lines()
            .map(|line| {
                line.split_whitespace()
                    .collect::<Vec<&str>>()
            })
            .collect();
        
        let mut mapping: Vec<MapData> = Vec::new();

        for line in lines {
            let addr_block = line.first().map(|s| s.to_string());
            let (start, end) = addr_block.as_ref().unwrap()
                .split_once("-")
                .expect("Failed to read address block");
            
            let perms = line.get(1).map(|s| s.to_string()).unwrap();

            mapping.push(MapData { 
                start:      u64::from_str_radix(start, 16).expect("failed to parse start addr."),
                end:        u64::from_str_radix(end, 16).expect("failed to parse end addr."),
                r:          perms.get(0..1) == Some("r"),
                w:          perms.get(1..2) == Some("w"), 
                x:          perms.get(2..3) == Some("x"),
                p:          perms.get(3..4) == Some("p"),
                s:          perms.get(3..5) == Some("s"),
                offset:     line.get(2).map(|s| s.to_string()).unwrap(), 
                device:     line.get(3).map(|s| s.to_string()).unwrap(), 
                inode:      line.get(4).map(|s| s.to_string()).unwrap(), 
                pathname:   line.get(5).map(|s| s.to_string()),
            }); 
        }

        self.mapping = mapping;
    }

    /// read a `T` from memory at `start_addr`
    pub fn read_mem<T: Pod>(&mut self, start_addr: u64) -> Result<T> {
        // reserve space for T
        let mut ret: MaybeUninit<T> = MaybeUninit::uninit();

        // go to start of addr block
        self.mem.seek(SeekFrom::Start(start_addr))?;

        // create an empty byte slice that points to ret
        let ptr = unsafe {
            core::slice::from_raw_parts_mut(
                ret.as_mut_ptr() as *mut u8, core::mem::size_of_val(&ret))
        };
        
        // fill the byte slice
        self.mem.read_exact(ptr)?;

        Ok(unsafe { ret.assume_init() })
    }

    // read a `T` from memory
    pub fn read_mem_slice<T: Pod>(&mut self, start_addr: u64, num_elems: usize) -> Result<Box<[T]>> {
        // reserve space for `num_elems` amount of T on the heap (Box)
        let mut ret: Box<[MaybeUninit<T>]> = Box::new_uninit_slice(num_elems);

        // go to start of addr block
        self.mem.seek(SeekFrom::Start(start_addr))?;
        
        // create an empty byte slice that points to ret
        let ptr = unsafe {
            core::slice::from_raw_parts_mut(
                ret.as_mut_ptr() as *mut u8, core::mem::size_of_val(&*ret))
        };

        // fill the byte slice
        self.mem.read_exact(ptr)?;

        Ok(unsafe { ret.assume_init() })
    }
}

pub unsafe trait Pod: Copy + Sync + Send + 'static {}

unsafe impl Pod for i8    {}
unsafe impl Pod for i16   {}
unsafe impl Pod for i32   {}
unsafe impl Pod for i64   {}
unsafe impl Pod for i128  {}
unsafe impl Pod for isize {}
unsafe impl Pod for u8    {}
unsafe impl Pod for u16   {}
unsafe impl Pod for u32   {}
unsafe impl Pod for u64   {}
unsafe impl Pod for u128  {}
unsafe impl Pod for usize {}
unsafe impl Pod for f32   {}
unsafe impl Pod for f64   {}

unsafe impl<const N: usize> Pod for [i8;    N] {}
unsafe impl<const N: usize> Pod for [i16;   N] {}
unsafe impl<const N: usize> Pod for [i32;   N] {}
unsafe impl<const N: usize> Pod for [i64;   N] {}
unsafe impl<const N: usize> Pod for [i128;  N] {}
unsafe impl<const N: usize> Pod for [isize; N] {}
unsafe impl<const N: usize> Pod for [u8;    N] {}
unsafe impl<const N: usize> Pod for [u16;   N] {}
unsafe impl<const N: usize> Pod for [u32;   N] {}
unsafe impl<const N: usize> Pod for [u64;   N] {}
unsafe impl<const N: usize> Pod for [u128;  N] {}
unsafe impl<const N: usize> Pod for [usize; N] {}
unsafe impl<const N: usize> Pod for [f32;   N] {}
unsafe impl<const N: usize> Pod for [f64;   N] {}
