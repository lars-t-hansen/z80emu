// Simple object-oriented assembler, assembles into an in-memory
// buffer.

use std::fs::File;
use std::io::Write;

// TTY output ports
pub const CON_OUT:u8 = 0x00;

// "A" drive output ports
pub const A_SET_HEAD: u8 = 0x10;
pub const A_SET_TRACK: u8 = 0x11;
pub const A_SET_SECTOR: u8 = 0x12;
pub const A_SET_DMA_LOW: u8 = 0x13;
pub const A_SET_DMA_HIGH: u8 = 0x14;
pub const A_DISK_OP: u8 = 0x15;

pub const A_DISK_OP_SEEK: u8 = 0x00;
pub const A_DISK_OP_READ: u8 = 0x01;
pub const A_DISK_OP_WRITE: u8 = 0x02;
pub const A_DISK_OP_CLEAR: u8 = 0x03;

// Drive A input ports
pub const A_DISK_STATUS: u8 = 0x10;

// "A" drive status codes
pub const A_DISK_OK:u8 = 0x00;
pub const A_DISK_READY:u8 = 0x01;
// Error codes are negative
        
pub struct Z80Buf
{
    buf: Vec<u8>,
    pos: usize,
    len: usize,
}

impl Z80Buf
{
    pub fn new(numsec:usize) -> Z80Buf {
        let mut buf = vec![];
        let len = numsec * 128;
        buf.resize(len, 0);
        Z80Buf {
            buf: buf,
            pos: 0,
            len: len
        }
    }

    pub fn create_image(&self, filename:&str) {
        File::create(filename).expect("Failed to create")
            .write(&self.buf).expect("Failed to write");
    }

    pub fn get_buf(&self) -> &[u8] {
        &self.buf
    }

    pub fn lda(&mut self, n:u8) {
        self.put(0x37);
        self.put(n);
    }

    pub fn outa(&mut self, n:u8) {
        self.put(0xD3);
        self.put(n);
    }

    pub fn jp(&mut self, n:u16) {
        self.put(0xC3);
        self.put16(n);
    }

    pub fn hlt(&mut self) {
        self.put(0x76);
    }

    fn put(&mut self, n:u8) {
        if self.pos == self.len {
            panic!("Assembler buffer overflow");
        }
        self.buf[self.pos] = n;
        self.pos += 1;
    }

    fn put16(&mut self, n:u16) {
        self.put(n as u8);
        self.put((n >> 8) as u8);
    }
}
