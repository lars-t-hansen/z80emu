// Execution engine proper: registers and memory; interpreter.

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
        self.af = 0;
        self.bc = 0;
        self.de = 0;
        self.hl = 0;
        self.pc = 0;
        self.sp = 0;
    }

    pub fn execute(&mut self) {
        let mut pc = self.pc;  // a macro, loadregs?
        let mem = self.mem;
        loop {
            let op = mem[pc as usize];
            pc += 1;
            match op {
                0x00 => /* NOP */ { }
                0x76 => /* HLT */ {
                    self.pc = pc;  // a macro, saveregs?
		    println!("Halted");
                    return;
                }
                _ => /* Unknown */ {
                    panic!("Unknown opcode {}", op);
                }
            }
        }            
    }
}
