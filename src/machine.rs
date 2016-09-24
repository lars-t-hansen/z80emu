use Z80Emu;

use console::Console;
use disk::Disk;

pub struct Machine
{
    console: Console,
    disk: Disk
}

impl Machine
{
    pub fn new(diskfile: &str) -> Machine {
        Machine { console: Console::start(),
                  disk: Disk::start(diskfile) }
    }
}

impl Z80Emu
{
    pub fn halt(&mut self) {
        self.machine.console.halt();
    }

    pub fn port_out(&mut self, port: u8, value: u8) {
        match port {
            // Output char
	    0 => self.machine.console.out_char(value),
            // Set head
            4 => self.machine.disk.set_head(value),
            // Set track
            5 => self.machine.disk.set_track(value),
            // Set DMA low
            6 => self.machine.disk.set_dma_low(value),
            // Set DMA high
            7 => self.machine.disk.set_dma_high(value),
            // Disk command
            8 => {
                match value {
                    0 => self.machine.disk.read_sector(),
                    1 => self.machine.disk.write_sector(),
                    2 => self.machine.disk.read_param(),
                    3 => self.machine.disk.seek(),
                    _ => panic!("Unknown disk command {}", value)
                }
            }
            // Set sector
            9 => self.machine.disk.set_sector(value),
	    _ => panic!("Unmapped output port {}", port)
        }
    }

    pub fn port_in(&mut self, port: u8) -> u8 {
        match port {	
            // Output char possible?  00 = yes, FF = no
            1 => if self.machine.console.out_ready() { 0 } else { 0xFF },
            // Input char available?  FF = yes, 00 = no
            2 => if self.machine.console.in_ready() { 0xFF } else { 0 },
            // Input char
            3 => self.machine.console.in_char(),
            // Disk status: 00 = idle, FF = busy, nn = idle, other status
            10 => self.machine.disk.status(),
            _ => panic!("Unmapped input port {}", port)
        }
    }
}
