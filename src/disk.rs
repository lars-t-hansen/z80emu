use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom};

pub struct Disk
{
    // Currently selected disk controller parameters
    pub head:   u8,
    pub track:  u8,
    pub sector: u8,
    pub dma_lo: u8,
    pub dma_hi: u8,

    // Result of operation
    pub result: u8,

    // Disk geometry
    max_head:   u8,
    max_track:  u8,
    max_sector: u8,

    // Seekable/readable/writable fixed-size backing store
    the_disk: File
}

pub fn make_disk(heads: u8, tracks: u8, sectors: u8) -> Result<Disk, io::Error}
{
    let the_disk = OpenOptions::new().read(true).write(true).open("disk.bin")?;
    Ok(Disk { head: 0, track: 0, sector: 0, dma_lo: 0, dma_hi: 0, result: 0,
              max_head:   heads-1,
              max_track:  tracks-1,
              max_sector: sectors-1,
              the_disk:   the_disk
    })
}

pub fn disk_op(op: u8, mem: &mut [u8], dsk:&mut Disk) {
    if !validate_disk_params(dsk) {
        dsk.result = 0xFF;
    } else {
        let offset = disk_translate(dsk) as u64;
        let dma = ((dsk.dma_hi as usize) << 8) | (dsk.dma_lo as usize);
        dsk.the_disk.seek(SeekFrom::Start(offset)).unwrap();
        match op {
            0x00 /* DISK_READ */ => {
                dsk.the_disk.read(&mut mem[dma .. dma + 128]).unwrap();
                dsk.result = 0x00;
            }
            _ => {
                dsk.result = 0xFF;
            }
        }
    }
}

fn disk_translate(dsk: &Disk) -> u32
{
    let sectors_per_track = dsk.max_sector as u32 + 1;
    let tracks_per_head = dsk.max_track as u32 + 1;
    let sectors_per_head = sectors_per_track * tracks_per_head;
    let sector_size = 128;
    
    dsk.head as u32 * sectors_per_head + dsk.track as u32 * sectors_per_track + dsk.sector as u32 * sector_size
}

fn validate_disk_params(dsk: &Disk) -> bool
{
    dsk.head <= dsk.max_head && dsk.track <= dsk.max_track && dsk.sector <= dsk.max_sector
}
