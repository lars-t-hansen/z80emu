use Z80Emu;

pub struct Z80 {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    pc: u16,
    sp: u16,
}

impl Z80 {
    pub fn new() -> Z80 {
        Z80 { af: 0, bc: 0, de: 0, hl: 0, pc: 0, sp: 0 }
    }
}

impl Z80Emu {
    pub fn install_rom(&mut self, rom: &[u8], addr: usize, romsiz: usize) {
        let mut i = 0;
        let mut k = addr & 65535;
        while i < romsiz {
            self.mem[k] = rom[i];
            k = (k + 1) & 65535;
            i = i + 1;
        }
    }

    pub fn reset(&mut self) {
        self.z80.af = 0;
        self.z80.bc = 0;
        self.z80.de = 0;
        self.z80.hl = 0;
        self.z80.pc = 0;
        self.z80.sp = 0;
    }

    pub fn execute(&mut self) {
        let mut pc = self.z80.pc;  // a macro, loadregs?
        let mut a = (self.z80.af >> 8) as u8;
        let mut zf = false;
        loop {
            let op = self.mem[pc as usize];  // a macro, nextbyte?
            println!("{} {}", pc, op);
            pc = pc.wrapping_add(1);
            match op {
                0x00 => /* NOP */ { }
                0x20 => /* JR NZ, offs */ {
                    let n = self.mem[pc as usize] as i8 as i16 as u16;
                    pc = pc.wrapping_add(1);
                    if !zf { pc = pc.wrapping_add(n); }
                }
                0x28 => /* JR Z, offs */ {
                    let n = self.mem[pc as usize] as i8 as i16 as u16;
                    pc = pc.wrapping_add(1);
                    if zf { pc = pc.wrapping_add(n); }
                }
                0x37 => /* LD A, n */ {
                    a = self.mem[pc as usize];
                    pc = pc.wrapping_add(1);
                }
                0x76 => /* HLT */ {
                    self.z80.pc = pc as u16;  // a macro, saveregs?
                    //self.z80.af = af;  // fixme
		    println!("Halted");
                    return;
                }
                0xD3 => /* OUT (n), A */ {
                    let n = self.mem[pc as usize];
                    pc = pc.wrapping_add(1);
                    self.port_out(n, a);
                }
                0xDB => /* IN A, (n) */ {
                    let n = self.mem[pc as usize];
                    pc = pc.wrapping_add(1);
                    a = self.port_in(n);
                }
                0xFE => /* CP A, n */ {
                    let n = self.mem[pc as usize];
                    pc = pc.wrapping_add(1);
                    zf = a == n;
                }                    
                _ => /* Unknown */ {
                    panic!("Unknown opcode {}", op);
                }
            }
        }            
    }
}
