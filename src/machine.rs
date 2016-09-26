use Z80Emu;

use console::Console;
use disk::Disk;

pub struct Machine
{
    console: Console,
    disk0: Disk
}

impl Machine
{
    pub fn new(diskfile: &str) -> Machine {
        Machine { console: Console::start(),
                  disk0: Disk::start(diskfile).unwrap() }
    }
}

// Port assignments:
//
//  0..3  Console
//    0 (in)  - poll output status (00=ready for output, FF=busy)
//    1 (out) - write char and set busy flag
//    2 (in)  - poll input available (FF=input available, 00=no input)
//    3 (in)  - read char and clear available flag
//
//  4..13 Disk 0
//    4 (in)  - poll disk status (00=idle+ok, FF=busy, nn=idle+error)
//    5 (out) - set head
//    6 (out) - set track
//    7 (out) - set sector
//    8 (out) - set transfer addr low
//    9 (out) - set transfer addr high
//   10 (out) - issue command
//                0 - read sector to buffer (reads: sector)
//                1 - write sector from buffer (reads: sector)
//                2 - seek (reads: head, track)
//                3 - copy memory to buffer (reads: transfer addr)
//                4 - copy buffer to memory (reads: transfer addr)
//   11 (in)  - get heads
//   12 (in)  - get tracks
//   13 (in)  - get sectors
//
//   Status must be polled after seek, read sector, and write sector;
//   no command must be issued while disk is busy.  (That copying is
//   explicit and not by DMA is actually a concession to the emulator
//   being written in safe Rust - DMA would be racy.)
//
//   Disk controller has a one-sector buffer for transfers.  The
//   buffer size is 128 bytes (fixed).

impl Z80Emu
{
    pub fn halt(&mut self) {
        self.machine.console.halt();
        self.machine.disk0.halt();
    }

    pub fn port_out(&mut self, port: u8, value: u8) {
        match port {
	    1 => self.machine.console.out_char(value),
            5 => self.machine.disk0.set_head(value),
            6 => self.machine.disk0.set_track(value),
            7 => self.machine.disk0.set_sector(value),
            8 => self.machine.disk0.set_addr_low(value),
            9 => self.machine.disk0.set_addr_high(value),
            10 => {
                match value {
                    0 => self.machine.disk0.read_sector(),
                    1 => self.machine.disk0.write_sector(),
                    2 => self.machine.disk0.seek_head_track(),
                    3 => self.machine.disk0.copy_from_addr(&mut self.mem),
                    4 => self.machine.disk0.copy_to_addr(&mut self.mem),
                    _ => panic!("Unknown disk command {}", value)
                }
            }
	    _ => panic!("Unmapped output port {}", port)
        }
    }

    pub fn port_in(&mut self, port: u8) -> u8 {
        match port {	
            0 => if self.machine.console.out_ready() { 0 } else { 0xFF },
            2 => if self.machine.console.in_ready() { 0xFF } else { 0 },
            3 => self.machine.console.in_char(),
            4 => self.machine.disk0.status(),
           11 => self.machine.disk0.heads as u8,
           12 => self.machine.disk0.tracks_per_head as u8,
           13 => self.machine.disk0.sectors_per_track as u8,
            _ => panic!("Unmapped input port {}", port)
        }
    }
}
