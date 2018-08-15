// A file backed spinning disk represents a spinning disk as a single file on
// the host system.
//
// Commands:
//   0x00 = SEEK
//   0x01 = READ
//   0x02 = WRITE

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom};

use devices::{SpinningDisk, SpinningDiskStatus};

pub struct FileBackedSpinningDisk
{
    // Currently selected disk controller parameters
    head:   u8,
    track:  u8,
    sector: u8,
    dma_lo: u8,
    dma_hi: u8,

    // Computed seek offset
    offset: u64,

    // Result of operation
    status: SpinningDiskStatus,

    // Disk geometry
    max_head:   u8,
    max_track:  u8,
    max_sector: u8,

    // Seekable/readable/writable fixed-size backing store
    the_disk:   File,
}

pub fn make(filename:&str, heads: u8, tracks: u8, sectors: u8)
            -> Result<FileBackedSpinningDisk, io::Error>
{
    let the_disk = try!(OpenOptions::new().read(true).write(true).open(filename));
    Ok(FileBackedSpinningDisk {
        head: 0, track: 0, sector: 0,
        dma_lo: 0, dma_hi: 0,
        offset:     0,
        status:     SpinningDiskStatus::SeekError,
        max_head:   heads-1,
        max_track:  tracks-1,
        max_sector: sectors-1,
        the_disk:   the_disk })
}

impl SpinningDisk for FileBackedSpinningDisk
{
    fn get_status(&mut self) -> SpinningDiskStatus { self.status }

    fn set_head(&mut self, n: u8) { self.head = n; }
    fn set_track(&mut self, n: u8) { self.track = n; }
    fn set_sector(&mut self, n: u8) { self.sector = n; }

    fn set_dma_high(&mut self, n: u8) { self.dma_hi = n; }
    fn set_dma_low(&mut self, n: u8) { self.dma_lo = n; }

    fn disk_operation(&mut self, op: u8, mem: &mut [u8]) {
        match op {
            0x00 => { self.seek(); }
            0x01 => { self.read_sector(mem); }
            0x02 => { self.write_sector(mem); }
            _    => { self.status = SpinningDiskStatus::OpError }
        }
    }
}

impl FileBackedSpinningDisk
{
    fn validate_params(&self) -> bool {
        self.head <= self.max_head && self.track <= self.max_track && self.sector <= self.max_sector
    }

    fn translate(&self) -> u32 {
        let sectors_per_track = self.max_sector as u32 + 1;
        let tracks_per_head = self.max_track as u32 + 1;
        let sectors_per_head = sectors_per_track * tracks_per_head;
        let sector_size = 128;
        
        self.head as u32 * sectors_per_head + self.track as u32 * sectors_per_track + self.sector as u32 * sector_size
    }

    fn seek(&mut self) {
        if !self.validate_params() {
            self.status = SpinningDiskStatus::SeekError;
            return;
        }
        self.offset = self.translate() as u64;
        self.status = SpinningDiskStatus::Ok;
    }
    
    fn read_sector(&mut self, mem: &mut [u8]) {
        if self.status != SpinningDiskStatus::Ok {
            return;
        }

        // We do the actual seek here, and not in seek(), since we may issue
        // multiple read operations for the same sector to different DMA
        // addresses, and we'd need to seek the file every time.

        match self.the_disk.seek(SeekFrom::Start(self.offset)) {
            Ok(_) => {
                let dma = ((self.dma_hi as usize) << 8) | (self.dma_lo as usize);
                // FIXME: Handle wraparound addresses
                match self.the_disk.read(&mut mem[dma .. dma + 128]) {
                    Ok(_) => { self.status = SpinningDiskStatus::Ok }
                    _     => { self.status = SpinningDiskStatus::ReadError }
                }
            }
            _ => { self.status = SpinningDiskStatus::ReadError; }
        }
    }

    fn write_sector(&mut self, mem: &mut [u8]) {
        if self.status != SpinningDiskStatus::Ok {
            return;
        }

        // IMPLEMENTME
        // NOTE wraparound addresses
        self.status = SpinningDiskStatus::WriteError;
    }
}

