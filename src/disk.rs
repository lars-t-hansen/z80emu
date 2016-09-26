use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Write, Seek, SeekFrom};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use std::thread::JoinHandle;

// Commands from main to disk

enum Command {
    Seek{head: u8, track: u8},
    Read{sector: u8},
    Write{sector: u8},
    Stop
}

// Result from disk to main / status of channel

const CMD_IDLE : usize = 0;
const CMD_BUSY : usize = 1;
const CMD_OK : usize = 2;

// Disk sector size is common knowledge, really

const SEC_SIZE : usize = 128;

// The backing file for a disk comprises a number of SEC_SIZE-byte sectors followed
// by some bytes that describe the disk geometry, currently the geo info is this:
//
//   heads: u8,
//   tracks: u8,
//   sectors: u8
//
// The number of sectors in the file must equal the product of the three values.
//
// The disk is laid out as a 3-dimensional row-major order array indexed by
// [head,track,sector].
    
// Disk parameter block constants

const DP_SIZE : usize = 3;        // Size of block in bytes
const DP_HEADS : usize = 0;       // Offset of number of heads
const DP_TRACKS : usize = 1;      // Offset of number of tracks
const DP_SECS : usize = 2;        // Offset of number of sectors

pub struct Disk {
    pub heads: usize,             // Constant: # of heads
    pub tracks_per_head: usize,   // Constant: # of tracks per head
    pub sectors_per_track: usize, // Constant: # of sectors per track

    state: u8,                    // IDLE/BUSY/BADPARAM

    head: u8,                     // Set by set_head
    track: u8,                    // Set by set_track
    sector: u8,                   // Set by set_sector
    addr: u16,                    // Set by set_addr_{high,low}

    cmd: Sender<Command>,         // Main -> disk commands
    result: Arc<AtomicUsize>,     // Cleared by main thread, set by disk thread
    buffer: Arc<Mutex<[u8; 128]>>,// Read/written by both threads
    disk_thread: JoinHandle<()>
}

// Status values of disk

const STAT_IDLE : u8 = 0x00;
const STAT_BADPARM : u8 = 0x01;
const STAT_BUSY : u8 = 0xFF;

// Data required by the worker thread

struct DiskThreadData {
    tracks_per_head: usize,
    sectors_per_track: usize,
    file: File,
    cmd_recv: Receiver<Command>,
    buffer: Arc<Mutex<[u8; 128]>>,
    result: Arc<AtomicUsize>
}

impl Disk
{
    pub fn start(diskfile:&str) -> io::Result<Disk> {
        let mut file = try!(OpenOptions::new().read(true).write(true).open(diskfile));

        let mut tmp = [0; DP_SIZE];
        let fsize = try!(file.seek(SeekFrom::End(-(DP_SIZE as i64))));
        try!(file.read(&mut tmp));

        let heads = tmp[DP_HEADS] as usize;
        let tracks_per_head = tmp[DP_TRACKS] as usize;
        let sectors_per_track = tmp[DP_SECS] as usize;

        assert!(fsize % (SEC_SIZE as u64) == 0);
        assert!((heads * tracks_per_head * sectors_per_track * SEC_SIZE) as u64 == fsize);

        let buffer : Arc<Mutex<[u8; 128]>> = Arc::new(Mutex::new([0; 128]));
        let result = Arc::new(AtomicUsize::new(CMD_IDLE));
        let (cmd_send, cmd_recv) = channel();

        let disk_thread = start_disk_thread(DiskThreadData {
            tracks_per_head: tracks_per_head,
            sectors_per_track: sectors_per_track,
            file: file,
            cmd_recv: cmd_recv,
            buffer: buffer.clone(),
            result: result.clone()
        });
        
        Ok(Disk {
            heads: heads,
            tracks_per_head: tracks_per_head,
            sectors_per_track: sectors_per_track,

            state: STAT_IDLE,

            head: 0,
            track: 0,
            sector: 0,
            addr: 0,

            result: result,
            cmd: cmd_send,
            buffer: buffer,
            disk_thread: disk_thread
        })
    }

    pub fn halt(&mut self) {
        self.cmd.send(Command::Stop).unwrap();
        // FIXME:  This fails in the borrow checker - why?
        // The usual disaster...
        //self.disk_thread.join();
    }

    pub fn set_head(&mut self, value:u8) {
        if self.state != STAT_BUSY {
            self.head = value;
        }
    }

    pub fn set_track(&mut self, value:u8) {
        if self.state != STAT_BUSY {
            self.track = value;
        }
    }

    pub fn set_sector(&mut self, value:u8) {
        if self.state != STAT_BUSY {
            self.sector = value;
        }
    }

    pub fn set_addr_low(&mut self, value:u8) {
        if self.state != STAT_BUSY {
            self.addr = (self.addr & 0xFF) | (value as u16);
        }
    }

    pub fn set_addr_high(&mut self, value:u8) {
        if self.state != STAT_BUSY {
            self.addr = (self.addr & 0x00FF) | ((value as u16) << 8);
        }
    }
    
    pub fn seek_head_track(&mut self) {
        if self.state != STAT_BUSY {
            self.state = STAT_BUSY;
            if !self.check_param() {
                self.state = STAT_BADPARM;
                return;
            }
            self.result.store(CMD_BUSY, Ordering::SeqCst);
            self.cmd.send(Command::Seek{head: self.head, track: self.track}).unwrap();
        }
    }

    pub fn read_sector(&mut self) {
        if self.state != STAT_BUSY {
            self.state = STAT_BUSY;
            if !self.check_param() {
                self.state = STAT_BADPARM;
                return;
            }
            self.result.store(CMD_BUSY, Ordering::SeqCst);
            self.cmd.send(Command::Read{sector: self.sector}).unwrap();
        }
    }

    pub fn write_sector(&mut self) {
        if self.state != STAT_BUSY {
            self.state = STAT_BUSY;
            if !self.check_param() {
                self.state = STAT_BADPARM;
                return;
            }
            self.result.store(CMD_BUSY, Ordering::SeqCst);
            self.cmd.send(Command::Write{sector: self.sector}).unwrap();
        }
    }

    pub fn copy_from_addr(&self, mem: &mut [u8; 65536]) {
        if self.state != STAT_BUSY {
            let mut k = self.addr;
            let buf = &mut self.buffer.lock().unwrap();
            for i in 0..SEC_SIZE-1 {
                buf[i] = mem[k as usize];
                k = k.wrapping_add(1);
            }
        }
    }

    pub fn copy_to_addr(&mut self, mem: &mut [u8; 65536]) {
        if self.state != STAT_BUSY {
            let mut k = self.addr;
            let buf = &mut self.buffer.lock().unwrap();
            for i in 0..SEC_SIZE-1 {
                mem[k as usize] = buf[i];
                k = k.wrapping_add(1);
            }
        }
    }

    pub fn status(&mut self) -> u8 {
        match self.result.load(Ordering::SeqCst) {
            CMD_OK => {
                self.state = STAT_IDLE;
                self.result.store(CMD_IDLE, Ordering::SeqCst);
            }
            CMD_BUSY => {}
            CMD_IDLE => {}
            _ => {}
        }
        return self.state;
    }

    fn check_param(&self) -> bool {
        return (self.head as usize) < self.heads && (self.track as usize) < self.tracks_per_head && (self.sector as usize) < self.sectors_per_track;
    }
}

fn start_disk_thread(mut dd: DiskThreadData) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut head_latched = 0;
        let mut track_latched = 0;
        loop {
            match dd.cmd_recv.recv().unwrap() {
                Command::Seek{head, track} => {
                    // Can't seek here because read/write advance the file pointer; we
                    // want consecutive reads/writes without intervening seek to
                    // access the same sector.  Thus seeking is done by read/write.
                    head_latched = head as usize;
                    track_latched = track as usize;
                    dd.result.store(CMD_OK, Ordering::SeqCst);
                }
                Command::Read{sector} => {
                    let sector_latched = sector as usize;
                    let address = dd.offs(head_latched, track_latched, sector_latched);
                    let buf = &mut *dd.buffer.lock().unwrap();
                    dd.file.seek(SeekFrom::Start(address as u64)).unwrap();
                    dd.file.read_exact(buf).unwrap();
                    dd.result.store(CMD_OK, Ordering::SeqCst);
                }
                Command::Write{sector} => {
                    let sector_latched = sector as usize;
                    let address = dd.offs(head_latched, track_latched, sector_latched);
                    let buf = &*dd.buffer.lock().unwrap();
                    dd.file.seek(SeekFrom::Start(address as u64)).unwrap();
                    dd.file.write_all(buf).unwrap();
                    dd.result.store(CMD_OK, Ordering::SeqCst);
                }
                Command::Stop => {
                    // So... presumably when the moved dd is destructed the file
                    // is closed.
                    return;
                }
            }
        }
    })
}

impl DiskThreadData {
    fn offs(&self, head: usize, track: usize, sector: usize) -> usize {
        ((head * self.tracks_per_head + track) * self.sectors_per_track + sector) * SEC_SIZE
    }
}

// For example:
// Boot rom loads 256-byte bootloader from (hd 0, trk 0, sector 0 and 1)
//
//  - set head 0
//  - set track 0
//  - wait for ready
//  - seek
//  - wait for ready
//  - set sector 0
//  - read sector
//  - wait for ready
//  - set address 100h
//  - copy to user
//  - set sector 1
//  - read sector
//  - wait for ready
//  - set address 180h
//  - copy to user
//  - jmp 100h
//
// Presumably there is some disk parameter block at the end of the
// bootloader to allow the loader to figure out how to load the OS.
