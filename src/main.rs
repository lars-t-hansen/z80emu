/**
 * Z80 emulator + machine model for a simplistic disk operating system.
 *
 * How it works (will eventually work):
 *
 * - We load a fixed bootstrap ("rom") into location 0 and reset the CPU
 * - The bootstrap loads a boot loader from head 0, track 0, sectors 0 and 1
 * - The boot loader loads the OS from subsequent sectors and installs it,
 *   and then calls the warmboot OS routine
 * - The OS warmboot loads the program 'C' and invokes it (the command processor)
 * - The command processor is interactive; naming a file loads that file
 *   as a program and runs it.
 *
 * See machine.rs for documentation of the i/o space.
 */

mod console;
mod disk;
mod machine;
mod z80;

use machine::Machine;
use z80::Z80;

use std::fs::File;
use std::io::Read;

struct Z80Emu {
    z80: Z80,
    mem: [u8; 65536],
    machine: Machine
}

fn main() {
    // TODO: allow romfile and diskfile to be overridden by command
    // line parameters or environment variables.

    let romfile = "rom.bin";
    let diskfile = "disk.bin";

    let mut emu = Z80Emu {
        z80: Z80::new(),
        mem: [0; 65536],
        machine: Machine::new(diskfile)
    };

    let rom = load_rom(romfile);
    emu.install_rom(&rom, 0, rom.len());

    emu.reset();
    emu.execute();

    emu.halt();
}

fn load_rom(filename: &str) -> Vec<u8> {
    let mut file = File::open(filename).unwrap();
    let mut bytes = Vec::<u8>::new();
    file.read_to_end(&mut bytes).unwrap();
    return bytes;
}
