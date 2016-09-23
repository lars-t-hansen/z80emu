mod machine;
mod z80;

use machine::Machine;
use z80::Z80;

use std::fs::File;
use std::io::Read;
use std::sync::Arc;

const ROMFILE : &'static str = "rom.bin";

fn main() {
    // TODO: allow romfile and diskfile to be overridden.

    let z80 = Z80::new();
    let machine = Machine::new(0);
    let rom = load_rom(ROMFILE);
    z80.install_rom(&rom, 0, rom.len());

    //machine.set_cpu(z80);

    z80.reset();
    z80.execute();

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
