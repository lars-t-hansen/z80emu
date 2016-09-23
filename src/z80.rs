use Z80Emu;

pub struct Z80 {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    pc: u16,
    sp: u16,
    mem: [u8; 65536]
}

impl Z80 {
    pub fn new() -> Z80 {
        Z80 { af: 0, bc: 0, de: 0, hl: 0, pc: 0, sp: 0, mem: [0; 65536] }
    }
}

impl Z80Emu {
    pub fn install_rom(&mut self, rom: &[u8], addr: usize, romsiz: usize) {
        let mut i = 0;
        let mut k = addr & 65535;
        let mem = &mut self.z80.mem;
        while i < romsiz {
            mem[k] = rom[i];
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
        loop {
            let op = self.z80.mem[pc as usize];  // a macro, nextbyte?
            pc = ((pc as usize) + 1) as u16;
            match op {
                0x00 => /* NOP */ { }
                0x37 => /* LD A, n */ {
                    a = self.z80.mem[pc as usize];
                    pc = ((pc as usize) + 1) as u16;
                }
                0x76 => /* HLT */ {
                    self.z80.pc = pc;  // a macro, saveregs?
                    //self.z80.af = af;  // fixme
		    println!("Halted");
                    return;
                }
                0xD3 => /* OUT (n), A */ {
                    let n = self.z80.mem[pc as usize];
                    pc = ((pc as usize) + 1) as u16;
                    self.out(n, a);
                }
                _ => /* Unknown */ {
                    panic!("Unknown opcode {}", op);
                }
            }
        }            
    }
}
