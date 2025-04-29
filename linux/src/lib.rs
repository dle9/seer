use log::{info, warn, debug, error};
use nix::{sys::{ptrace, wait}, unistd::Pid};
use std::fs::{File, read_to_string};
use std::io::{Read, Seek, SeekFrom};
use std::mem::MaybeUninit;
use anyhow::Result;

/// for /proc/<pid>/maps
#[derive(Debug)]
struct Maps {
    /// start addr of the mapping address block.
    start:      usize,

    /// end addr of the mapping address block.
    end:        usize,

    /// permissions: `r`, `w`, `x`, `p` (private), `s` (shared).
    r:          bool,
    w:          bool,
    x:          bool,
    p:          bool,
    s:          bool,

    /// offset within the file for file mappings.
    offset:     usize,

    /// device id `<major>:<minor>`.
    device:     String,

    /// inode for file mappings.
    inode:      String,

    /// filesystem path for file mappings.
    pathname:   Option<String>,
}


/// for /proc/<pid>/mem
pub struct Mem {
    /// pid of target process
    pid:        Pid,

    /// vector holding the entire virtual addr mappings from /proc/pid/maps
    mapping:   Vec<Maps>,
}

impl Mem {
    pub fn new() -> Result<Self> {
        Ok(Self {
            pid: Pid::from_raw(0),
            mapping: Vec::new(),
        })
    }

    pub fn set_pid(&mut self, _pid: i32) {
        self.pid = Pid::from_raw(_pid);
    }

    /// dump memory of linux process
    pub fn dump(&mut self) -> Result<()> {
        ptrace::attach(self.pid).expect("failed to attach to pid.");

        match wait::waitpid(self.pid, None) { 
            Ok(wait::WaitStatus::Stopped(_, _)) => {
                info!("ptrace::attach({})", self.pid);

                // read from /proc/pid/maps and put into self.mapping
                self.load_mapping();
                    
                // create a copy of the mapping
                let old_mapping: Vec<Maps> = std::mem::take(&mut self.mapping);

                // read from /proc/pid/mem
                self.read_memory(&old_mapping);

                // restore the old mapping and detach from the ptrace
                self.mapping = old_mapping;
                ptrace::detach(self.pid, None).expect("failed to detach pid.");
                info!("ptrace::detach({})", self.pid);
            },

            Err(e) => error!("waitpid error {}", e),

            _ => ()
        };

        Ok(())
    }    

    /// parse data from /proc/<pid>/maps
    fn load_mapping(&mut self) {
        let raw = read_to_string(format!("/proc/{}/maps", self.pid))
            .expect("Failed to read mapping");

        let lines: Vec<Vec<&str>> = raw
            .lines()
            .map(|line| {
                line.split_whitespace()
                    .collect::<Vec<&str>>()
            })
            .collect();
        
        let mut mapping: Vec<Maps> = Vec::new();

        for line in lines {
            let addr_block = line.first().map(|s| s.to_string());
            let (start, end) = addr_block.as_ref().unwrap()
                .split_once("-")
                .expect("Failed to read address block");
            
            let perms = line.get(1).map(|s| s.to_string()).unwrap();
            
            let offset = line.get(2).map(|s| s.to_string()).unwrap();

            mapping.push(Maps { 
                start:      usize::from_str_radix(start, 16).expect("failed to parse start addr."),
                end:        usize::from_str_radix(end, 16).expect("failed to parse end addr."),
                r:          perms.get(0..1) == Some("r"),
                w:          perms.get(1..2) == Some("w"), 
                x:          perms.get(2..3) == Some("x"),
                p:          perms.get(3..4) == Some("p"),
                s:          perms.get(3..5) == Some("s"),
                offset:     usize::from_str_radix(&offset, 16).expect("failed to parse offset."), 
                device:     line.get(3).map(|s| s.to_string()).unwrap(), 
                inode:      line.get(4).map(|s| s.to_string()).unwrap(), 
                pathname:   line.get(5).map(|s| s.to_string()),
            }); 
        }

        self.mapping = mapping;
    }

    /// read a `T` from memory at `start_addr`
    fn read_memory(&mut self, mapping: &[Maps]) {
        // iter over the mapping and find strings
        for map in mapping.iter() {
            // only continue if readable
            if !map.r {
                continue;
            }
            
            // dont read these
            if let Some(file) = &map.pathname {
                if file.starts_with("/usr") {
                    continue;
                }
            }

            // for logging
            Mem::display_mapping(map);

            // read the entire memory region
            let num_elems = map.end - map.start;
            let start_addr = map.start as u64;
            if let Ok(data) = self.read_mem_slice::<u8>(start_addr, num_elems, 0) {
                let mut i = 0;
                while i < data.len() {
                    // check valid ascii
                    let string_start = i;
                    let mut string_len = 0;
                    while i < data.len() && data[i].is_ascii() && data[i] >= 32 && data[i] <= 126 {
                        string_len += 1;
                        i += 1;
                    }

                    // check length 
                    if string_len >= 4 {
                        let string_data = String::from_utf8_lossy(&data[string_start..string_start + string_len]);
                        info!("0x{:x}: {}", start_addr + string_start as u64, string_data);
                    }

                    if i == string_start {
                        i += 1;
                    }
                }
            }
        }
    }

    // read a `T` from memory
    fn read_mem_slice<T: Pod>(&mut self, start_addr: u64, num_elems: usize, offset: usize) -> Result<Box<[T]>> {
        // reserve uninitialized (MaybeUninit) space for `num_elems` amount of T on the heap (Box)
        let num_elems = num_elems.saturating_sub(offset);
        let mut ret: Box<[MaybeUninit<T>]> = Box::new_uninit_slice(num_elems);

        // go to start of addr block
        let mut mem_file = File::open(format!("/proc/{}/mem", self.pid))?;
        mem_file.seek(SeekFrom::Start(start_addr))?;
        
        // create an empty byte slice that points to ret
        let ptr = unsafe {
            core::slice::from_raw_parts_mut(
                ret.as_mut_ptr() as *mut u8, core::mem::size_of_val(&*ret))
        };

        // fill the byte slice
        match mem_file.read_exact(ptr) {
            Ok(()) => Ok(unsafe { ret.assume_init() }),
            Err(e) => {
                warn!("Failed to read memory at {:x} (+{:x}): {}", start_addr, offset, e);
                Err(e.into())
            }
        }
    }

    fn display_mapping(mapping: &Maps) {
        let mut perms = String::with_capacity(4);
            perms.push(if mapping.r     { 'r' } else { '-' });
            perms.push(if mapping.w     { 'w' } else { '-' });
            perms.push(if mapping.x     { 'x' } else { '-' });
            perms.push(if mapping.p     { 'p' }
                     else if mapping.s  { 's' }
                     else               { '-' });

        debug!("{:x}-{:x} {} {:x} {} {} {}", 
            mapping.start, mapping.end,
            perms,
            mapping.offset,
            mapping.device,
            mapping.inode,
            if let Some(pathname) = &mapping.pathname { pathname } else { "" }
        );
    }
}

/// Plain Old Data type
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
