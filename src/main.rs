/**
 * Z80 emulator + machine model for a simplistic disk operating system.
 *
 * How it works (will eventually work):
 *
 * - We load a fixed bootstrap ("rom") into location 0 and reset the CPU
 * - The bootstrap loads a boot loader from head 0, track 0, sectors 0 and 1
 * - The boot loader loads the OS from subsequent sectors and installs it,
 *   and then calls the warmboot OS routine
 * - The OS warmboot loads the command processor
 * - The command processor is interactive
 *
 * See machine.rs for documentation of the i/o space.
 */

mod console;
mod disk;
mod machine;
mod z80;

use console::Console;
use disk::Disk;
use z80::Z80;

struct Z80Emu {
    z80: Z80,
    mem: [u8; 65536],
    console: Console,
    disk0: Disk
}

fn main() {
    // TODO: allow romfile and diskfile to be overridden by command
    // line parameters or environment variables.

    // TODO: allow there to be multiple disks, depending on how many
    // disk files are supplied on the command line.

    let romfile = "rom.bin";
    let diskfile = "disk.bin";

    let mut emu = Z80Emu {
        z80: Z80::new(),
        mem: [0; 65536],
        console: Console::start(),
        disk0: Disk::start(diskfile)
    };

    emu.load_rom(romfile, 0);

    emu.reset();
    emu.execute();

    emu.console.halt();
    emu.disk0.halt();
}
