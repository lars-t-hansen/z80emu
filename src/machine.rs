/**
 * Z80 emulator + machine model for a simplistic disk operating system.
 *
 * How it works:
 *
 * - We load a fixed bootstrap ("rom") into location 0 and reset the CPU
 * - The bootstrap loads a boot loader from sector 0
 * - The boot loader loads the OS from subsequent sectors and installs it,
 *   and then calls the warmboot OS routine
 * - The OS warmboot loads the program 'C' and invokes it (the command processor)
 * - The command processor is interactive; naming a file loads that file
 *   as a program and runs it.
 *
 * There is a console:
 *
 * - OUT (0), r to write a character to the console
 * - IN  r, (1) to poll whether the character has been sent (0=ready, 0ffh=busy)
 * - IN  r, (2) to poll whether a character is available (0ffh=available, 0=not)
 * - IN  r, (3) to read a character (ready or not), clears the available flag
 *
 * There is a single disk-like (but linear) storage unit:
 *
 * - OUT (4), r to set low byte of disk block address
 * - OUT (5), r to set high byte of disk block address
 * - OUT (6), r to set low byte of memory block address
 * - OUT (7), r to set high byte of memory block address
 * - OUT (8), r to set and perform operation: 0=read, 1=write
 * - IN  r, (9) to poll for operation completeness; the
 *   data are transfered by DMA without involving the CPU.
 *   Value reads as 0 if ready, FFh if not ready
 * - IN  r, (10) to get completion code.  Zero if OK, otherwise
 *   something else.
 *
 * Polling / busy-wait is bogus but closer to reality than nothing.
 *
 * Soon, we want disk/io and console i/o to run on separate threads, and to support
 * interrupts when disk requests are done, when chars have become available, and
 * perhaps even when chars have been written.  BIOS code would then halt to wait for
 * interrupts.  (Let's hope the halt instruction re-enables interrupts or there
 * could be a race if the device is fast relative to the cpu.)  There would even be
 * separate seek (set disk, set head, set track) and read/write ops.
 */

use z80::Z80;

use std::fs::File;

pub struct Machine<'a>
{
    // CPU, we'll need this to access memory for DMA
    z80: Option<&'a mut Z80>,

    // Console
    char_available: u8,        // set to 0ffh when char available, cleared when char read
    the_char: u8,              // the available char, until the next one arrives
    char_written: u8,          // set to 0 when char has been successfully written, 0ffh while busy

    // Simplistic block device (array of blocks)
    nblocks: u32,              // number of blocks on block device
    //block_device: &'a mut File,    // backing store for block device
    disk_ready: u8,            // set to 0 when disk is idle / io is complete, 0ffh while busy
    disk_result: u8,           // result of last io operation, 0=ok otherwise some error
    disk_blockaddress_lo: u8,  // set by out()
    disk_blockaddress_hi: u8,  // set by out()
    disk_memoryaddress_lo: u8, // set by out()
    disk_memoryaddress_hi: u8  // set by out()
}

impl<'a> Machine<'a>
{
    pub fn new(/*device_: &'a mut File,*/ nblocks_: u32) -> Machine<'a> {
        Machine { z80: None,

                  char_available: 0,
	          the_char: 0,
		  char_written: 0,

                  nblocks: nblocks_,
	          //block_device: device_,
                  disk_ready: 0,
                  disk_result: 0,
                  disk_blockaddress_lo: 0,
                  disk_blockaddress_hi: 0,
                  disk_memoryaddress_lo: 0,
                  disk_memoryaddress_hi: 0 }
    }

    pub fn set_cpu(&mut self, z80:&'a mut Z80) {
       self.z80 = Some(z80);
    }

/*
    pub fn out(&self, port: u8, value: u8) {
        match port {
	    0 => {
                iosys.putchar(value);
                self.char_written = 0;   // ready
            }
	    4 => {
	        self.disk_blockaddress_lo = value;
            }
	    5 => {
	        self.disk_blockaddress_hi = value;
            }
    	    6 => {
                self.disk_memoryaddress_lo = value;
            }
	    7 => {
                self.disk_memoryaddress_hi = value;
            }
	    8 => {
	        u16 blockno = ((u16)self.disk_blockaddress_hi << 8) | self.disk_blockaddress_lo;
                u16 address = ((u16)self.disk_memoryaddress_hi << 8) | self.disk_memoryaddress_lo;
                size_t nbytes;

                self.disk_ready = 0xFF;

		// FIXME: unreasonable to check this here, should be in BIOS
		// We could however just mask with nblocks-1.
                if blockno >= self.nblocks {
                    self.disk_result = 1;
		    self.disk_ready = 0;
                    return;
                }

		// FIXME: unreasonable to check this here, should be in BIOS
		// We can't mask the address with 255.  We could wrap around,
		// which seems most reasonable, it might be what a real DMA would do.
                if (u32)address + 256 >= 65536 {
                    self.disk_result = 2;
		    self.disk_ready = 0;
                    return;
                }

                if value == 0 || value == 1 {
                    iosys.seek(block_device, (long)blockno*256, SEEK_SET);
                }
                if value == 0 {
                    nbytes = iosys.read(z80->M + address, 1, 256, block_device);
                }
                else if value == 1 {
                    nbytes = iosys.write(z80->M + address, 1, 256, block_device);
                }
                else if value == 2 {
                    // Disk parameter read
		    // FIXME
                    //z80->M[address] = nblocks;
                }
            }
            else {
                panic!("Bad disk I/O operation {?:}\n", value);
            }

            if (value == 0 || value == 1) && nbytes != 256 {
                self.disk_result = 3;
		self.disk_ready = 0;
                return;
            }

            self.disk_result = 0;
            self.disk_ready = 0;
            break;
        }
	_ => {
	    panic!("Unmapped output port {?:}\n", port);
        }
    }

    pub fn in(&self, port: u8): u8 {
        match port {	
            1 => {
                return self.char_written;
            }
            2 => {
                if (self.char_available)
                    return self.char_available;
                self.the_char = iosys.blocking_getchar();
                self.char_available = 255;
                return self.char_available;
            }
            3 => {
                self.char_available = 0;
                return self.the_char;
            }
            9 => {
                return self.disk_ready;
            }
            10 => {
                return self.disk_result;
            }
            _ => {
                panic!("Unmapped input port {:?}", port);
            }
        }
    }

    pub fn out(&self, port: u8, value: u8):(int,int) {
        out(port, value);
        return (-1, -1);  // FIXME, not correct
    }

    pub fn in(&self, port: u8):(u8,(int,int)) {
        return (in(port),(-1,-1));
    }
*/
}
