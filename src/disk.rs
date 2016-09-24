pub struct Disk {
}

impl Disk {
    pub fn start(diskfile:&str) -> Disk {
        Disk {}
    }

    pub fn halt(&mut self) {
    }

    pub fn set_head(&mut self, value:u8) {
    }

    pub fn set_track(&mut self, value:u8) {
    }

    pub fn set_sector(&mut self, value:u8) {
    }

    pub fn set_dma_low(&mut self, value:u8) {
    }

    pub fn set_dma_high(&mut self, value:u8) {
    }
    
    pub fn seek(&mut self) {
    }

    pub fn read_sector(&mut self) {
    }

    pub fn write_sector(&mut self) {
    }

    pub fn read_param(&mut self) {
    }

    // 00 = idle, status clear
    // FF = busy
    // xx = other status

    pub fn status(&mut self) -> u8 {
        return 0;
    }
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

*/
//}

// The disk system has a single thread probably, that accepts commands
// on a channel and returns responses in various ways.  While the disk
// is doing its thing the CPU can continue.  The disk should vector an
// interrupt when it's done, though it will also have a polling interface.

// Command classes:
//
// Status queries
// Read command
// Write command
// Seek command (set head / track / sector)

// The disk is backed by a regular file of fixed size.  We can do
// synchronous commands on that file.  There will be no background
// activity or buffering in the disk (yet) - it's old technology.
// So for simplicity we will implement disk ops as file ops, with
// seek / read / write, and we will flush after each write.

// Not sure how to handle DMA, since that is inherently racy.  For now,
// pass a buffer back and forth for the data.

// Boot rom loads 256-byte bootloader from (hd 0, trk 0, sector 0 and 1)
//
//  - set head 0
//  - set track 0
//  - wait for ready
//  - seek
//  - wait for ready
//  - read sector 0 to address 100h
//  - wait for ready
//  - read sector 1 to address 180h
//  - wait for ready
//  - jmp 100h
//
// Presumably there is some disk parameter block at the end of the
// bootloader to allow the loader to figure out how to load the OS.



    /*
    struct stat info;

    if (stat(diskfile, &info) != 0) {
        fprintf(stderr, "Could not open disk image file %s\n", diskfile);
        exit(1);
    }
    if (info.st_size % 256 != 0) {
        fprintf(stderr, "Disk image file size is not divisible by block size\n");
        exit(1);
    }
    unsigned nblocks = info.st_size / 256;
    FILE* block_device = fopen(diskfile, "rb+");
    if (block_device == NULL) {
        fprintf(stderr, "Could not open block device file\n");
        exit(1);
    }
    */

        // TODO:
        // - spin up a thread to handle the "disk"
        // - for now, let the number of blocks be a constant known by that thread
        // - on startup the thread needs to load the data
        // - a disk operation is then:
        //   - a command to spin up the disk
        //   - an interrupt when the disk is spun up
        //   - commands to set parameters
        //   - a command to perform the operation
        //   - an interrupt when the operation is done, or busy-wait (depending on parameters)
        // - when data are written the thread needs to write them, probably
        //   asynchronously, with interrupt on done
        // - Presumably a lot of these disk data are in a monitor somehow
        //
        // TODO:
        // - Console should be a thread too.


    /*
    // Simplistic block device (array of blocks)
    disk_ready: u8,            // set to 0 when disk is idle / io is complete, 0ffh while busy
    disk_result: u8,           // result of last io operation, 0=ok otherwise some error
    disk_blockaddress_lo: u8,  // set by out()
    disk_blockaddress_hi: u8,  // set by out()
    disk_memoryaddress_lo: u8, // set by out()
    disk_memoryaddress_hi: u8  // set by out()
     */


                  /*disk_ready: 0,
                  disk_result: 0,
                  disk_blockaddress_lo: 0,
                  disk_blockaddress_hi: 0,
                  disk_memoryaddress_lo: 0,
                  disk_memoryaddress_hi: 0 */

        // Stop the disk.  Flushing its backing store should be done as part
        //  of programmatic action, not here, but this should kill the
        //  threads at least.
        // Stop the console: kill the threads.
