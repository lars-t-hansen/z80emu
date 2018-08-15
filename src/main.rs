mod z80;
mod devices;
mod rust_console_io;
mod file_backed_spinning_disk;

use std::fs::OpenOptions;
use std::io::{self, Read};

use z80::StopReason;
use devices::{TTY, SpinningDisk};

// Disk parameters.
const A_HEADS   : u8 = 1;       // Single sector
const A_TRACKS  : u8 = 1;       //   disk for
const A_SECTORS : u8 = 1;       //     testing

struct Machine<'a> {
    tty:   &'a mut TTY,
    dsk_a: &'a mut SpinningDisk,
    // Many more concrete devices here
}

fn main() -> Result<(), io::Error>
{
    let mut _dsk_a = try!(file_backed_spinning_disk::make("disk.bin", A_HEADS, A_TRACKS, A_SECTORS));
    let mut _tty = rust_console_io::make();

    let mut m = Machine {
        tty:   &mut _tty,
        dsk_a: &mut _dsk_a,
    };

    let mut cpu = z80::make(/*pc=*/ 0x0000);

    // 128 bytes of boot ROM at 0000h.
    try!(setup_boot_rom(&mut cpu.mem[0..128]));

    loop {
        z80::run(&mut cpu);
        match cpu.stop_reason {
            StopReason::Halt => {
                break;
            }
            StopReason::Poll => {
            }
            StopReason::In => {
                cpu.a = port_in(cpu.port_addr, &mut m);
            }
            StopReason::Out => {
                port_out(cpu.port_addr, cpu.a, &mut cpu.mem, &mut m);
            }
            StopReason::Illegal => {
                panic!("Illegal instruction");
            }
        }
    }
    Ok(())
}

fn setup_boot_rom(mem: &mut [u8]) -> Result<(), io::Error>
{
    try!(OpenOptions::new().read(true).open("rom.bin")?.read(mem));
    Ok(())
}

fn port_out(port: u8, value: u8, mem: &mut [u8], m: &mut Machine)
{
    match port {
        0x00 => /* CHAR_OUT (n) */ { m.tty.put_nonblocking(value); }

        // "A" drive is a spinning disk
        0x10 => /* SET_HEAD (n) */ { m.dsk_a.set_head(value); }
        0x11 => /* SET_TRACK (n) */ { m.dsk_a.set_track(value); }
        0x12 => /* SET_SECTOR (n) */ { m.dsk_a.set_sector(value); }
        0x13 => /* SET_DMA_LOW (n) */ { m.dsk_a.set_dma_low(value); }
        0x14 => /* SET_DMA_HIGH (n) */ { m.dsk_a.set_dma_high(value); }
        0x15 => /* DISK_OP (n) */ { m.dsk_a.disk_operation(value, mem); }

        _ => /* Unknown */ { panic!("Unassigned output port {}", port); }
    }
}

fn port_in(port: u8, m: &mut Machine) -> u8
{
    match port {
        0x00 => /* CHAR_IN */ { m.tty.get_nonblocking() }
        0x01 => /* CHAR_AVAIL => 00h or FFh */ { m.tty.poll_nonblocking() }

        // "A" drive is a spinning disk
        0x10 => /* DISK_RESULT */ { m.dsk_a.get_status() as u8 }

        _ => /* Unknown */ { panic!("Unassigned input port {}", port); }
    }
}
