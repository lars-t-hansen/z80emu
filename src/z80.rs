use Z80Emu;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct Z80 {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    pc: u16,
    sp: u16,
    interrupt: Arc<AtomicUsize>
}

impl Z80
{
    pub fn new(interrupt: Arc<AtomicUsize>) -> Z80 {
        Z80 { af: 0, bc: 0, de: 0, hl: 0, pc: 0, sp: 0, interrupt: interrupt }
    }
}

impl Z80Emu
{
    pub fn reset(&mut self) {
        self.z80.af = 0;
        self.z80.bc = 0;
        self.z80.de = 0;
        self.z80.hl = 0;
        self.z80.pc = 0;
        self.z80.sp = 0;
    }

    pub fn execute(&mut self) {
        let mut pc = self.z80.pc;
        let mut a = (self.z80.af >> 8) as u8;
        let mut zf = false;

        macro_rules! byte {
            () => {{
                let n = self.mem[pc as usize];
                pc = pc.wrapping_add(1);
                n
            }}
        }

        macro_rules! byte_sext {
            () => ( byte!() as i8 as i16 as u16 )
        }
        
        macro_rules! word {
            () => {{
                let lo = byte!() as u16;
                let hi = byte!() as u16;
                (hi << 8) | lo
            }}
        }

        loop {
            // Service interrupts
            loop {
                // TODO: if interrupts disabled then don't look
                //
                // TODO: There needs to be a PIC device that will just consume interrupts
                // from the devices until it's set up to deliver them to the CPU.
                // That should be modeled as a thread that has one atomic that it shares
                // with the devices and one that it shares with the CPU.
                //
                // The BIOS programs the PIC once it's up and has set up vectors.

                let intr = self.z80.interrupt.load(Ordering::Relaxed);
                if intr == 0 { break; }
                // TODO: service interrupt.  The interrupt number is an index into a vector
                // in low memory that contains an address to jump to.  Presumably we push
                // the PC (and flags?) and disable interrupts and jump.
                println!("Interrupt: {}", intr & 7);
                self.z80.interrupt.store(0, Ordering::Relaxed);
            }

            // Execute one instruction
            let op = byte!();
            match op {
                0x00 => /* NOP */ { }
                0x20 => /* JR NZ, offs */ {
                    let n = byte_sext!();
                    if !zf { pc = pc.wrapping_add(n); }
                }
                0x28 => /* JR Z, offs */ {
                    let n = byte_sext!();
                    if zf { pc = pc.wrapping_add(n); }
                }
                0x37 => /* LD A, n */ {
                    a = byte!();
                }
                0x76 => /* HLT */ {
                    self.z80.pc = pc;
                    //self.z80.af = af;       // FIXME
		    println!("Halted");
                    return;
                }
                0xC3 => /* JP pq */ {
                    // We get a spurious unused assignment error here because
                    // the last update of the PC by 'word' is unnecessary, but
                    // squelching it with #[allow(unused_assignments)] is not
                    // supported except at the function level.
                    let npc = word!();
                    pc = npc;
                }
                0xD3 => /* OUT (n), A */ {
                    let n = byte!();
                    self.port_out(n, a);
                }
                0xDB => /* IN A, (n) */ {
                    let n = byte!();
                    a = self.port_in(n);
                }
                0xFE => /* CP A, n */ {
                    let n = byte!();
                    zf = a == n;
                    // FIXME: more flags
                }                    
                _ => /* Unknown */ {
                    panic!("Unknown opcode {}", op);
                }
            }
        }            
    }
}
