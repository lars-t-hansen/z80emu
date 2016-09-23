/**
 * Z80 emulator + machine model for a simplistic disk operating system.
 *
 * How it works (will eventually work):
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
 * We want disk/io and console i/o to run on separate threads, and to
 * support interrupts when disk requests are done, when chars have
 * become available, and perhaps even when chars have been written.
 * BIOS code would then halt to wait for interrupts.  There would even
 * be separate seek (set disk, set head, set track) and read/write
 * ops.
 */

mod machine;
mod z80;

use machine::Machine;
use z80::Z80;

use std::fs::File;
use std::io::Read;

// Arguably the wrong pattern: The "machine" and the "cpu" are
// separate ideas and there's no reason they should be in the same
// structure, but they need to reference each other and Rust prevents
// them from being separate structures that link to each other.

struct Z80Emu {
    z80: Z80,
    machine: Machine
}

fn main() {
    // TODO: allow romfile and diskfile to be overridden by command
    // line parameters or environment variables.

    let romfile = "rom.bin";
    let diskfile = "disk.bin";

    let mut emu = Z80Emu {
        z80: Z80::new(),
        machine: Machine::new(diskfile) };

    let rom = load_rom(romfile);
    emu.install_rom(&rom, 0, rom.len());

    emu.reset();
    emu.execute();

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

    /*
    File disk = ...;
    u32 nblocks = ...;

    Machine m = Machine::new(disk, nblocks);

    m.loadROM(romfile);

    Machine m(block_device, nblocks);
    rom_t roms[1];
    roms[0].address = 0;
    roms[0].size = sizeof(romImage);
    roms[0].image = romImage;
    Z80 z80(m, 1, roms);
    m.setCPU(&z80);

    z80.reset();
    z80.execute();

    // Not so good, want to ensure this is done.
    fclose(block_device);
    */
}

fn load_rom(filename: &str) -> Vec<u8>
{
    match File::open(filename) {
        Ok(mut f) => {
	    let mut bytes = Vec::<u8>::new();
	    match f.read_to_end(&mut bytes) {
	        Ok(_) => bytes,
                Err(err) => {
                    panic!("Can't read ROM file {}: {:?}", filename, err);
                }
            }
        }
	Err(err) => {
	    panic!("Can't open ROM file {}: {:?}", filename, err);
        }
    }
}
