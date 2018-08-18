pub struct Z80
{
    // 64KB of RAM
    pub mem: [u8; 65536],

    // Other state
    pub stop_reason: StopReason,
    pub port_addr: u8,

    // Standard registers
    pub pc: u16,
    pub sp: u16,
    pub ix: u16,
    pub iy: u16,
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    // Alternate registers
    a_alt: u8, f_alt: u8, b_alt: u8, c_alt: u8, d_alt: u8, e_alt: u8, h_alt: u8, l_alt: u8,
}

pub enum StopReason {
    Halt,                       // HLT executed
    Poll,                       // Timeslice expired
    Out,                        // OUT executed
    In,                         // IN executed
    Illegal                     // Illegal opcode and/or argument
}

pub fn make(pc:u16) -> Z80 {
    // TODO: On RESET, the pc is zero but the other registers are all random,
    // and it would be useful to set them to random values here.

    Z80 {
        mem: [0; 65536], pc: pc, sp: 0, ix: 0, iy: 0,
        stop_reason: StopReason::Poll,
        port_addr: 0,
        a: 0, f: 0, b: 0, c: 0, d: 0, e: 0, h: 0, l: 0,
        a_alt: 0, f_alt: 0, b_alt: 0, c_alt: 0, d_alt: 0, e_alt: 0, h_alt: 0, l_alt: 0
    }
}

// Flag bits in the flag registers

const CARRY_FLAG: u8 = 0x01;
const CARRY_SHIFT: u8 = 0;

const NEG_FLAG: u8 = 0x02;
const NEG_SHIFT: u8 = 1;

const OVERFLOW_FLAG: u8 = 0x04;
const OVERFLOW_SHIFT: u8 = 2;

const PARITY_FLAG: u8 = 0x04;
const PARITY_SHIFT: u8 = 2;

const HALF_FLAG: u8 = 0x10;
const HALF_SHIFT: u8 = 4;

const ZERO_FLAG: u8 = 0x40;
const ZERO_SHIFT: u8 = 6;

const SIGN_FLAG: u8 = 0x80;
const SIGN_SHIFT: u8 = 7;

const UNUSED_FLAGS: u8 = 0x28;

pub fn run(z80: &mut Z80, mut timeslice: usize) {
    let mem = &mut z80.mem;
    let mut pc = z80.pc;
    let mut sp_ = z80.sp;
    let mut ix_ = z80.ix;
    let mut iy_ = z80.iy;
    let mut a = z80.a;
    let mut f = z80.f;
    let mut b = z80.b;
    let mut c = z80.c;
    let mut d = z80.d;
    let mut e = z80.e;
    let mut h = z80.h;
    let mut l = z80.l;

    // 16-bit register operations

    macro_rules! get16 {
        ($hi:ident, $lo:ident) => {
            (($hi as u16) << 8) | ($lo as u16)
        }
    }
    macro_rules! set16 {
        ($hi:ident, $lo:ident, $v:ident) => {
            $lo = ($v & 0xFF) as u8;
            $hi = (($v >> 8) & 0xFF) as u8;
        }
    }

    macro_rules! bc { () => { get16!(b, c) } }
    macro_rules! de { () => { get16!(d, e) } }
    macro_rules! hl { () => { get16!(h, l) } }
    macro_rules! ix { () => { ix_ } }
    macro_rules! iy { () => { iy_ } }
    macro_rules! sp { () => { sp_ } }

    macro_rules! set_bc { ($v:ident) => { set16!(b, c, $v); } }
    macro_rules! set_de { ($v:ident) => { set16!(d, e, $v); } }
    macro_rules! set_hl { ($v:ident) => { set16!(h, l, $v); } }
    macro_rules! set_ix { ($v:ident) => { ix_ = $v; } }
    macro_rules! set_iy { ($v:ident) => { iy_ = $v; } }
    macro_rules! set_sp { ($v:ident) => { sp_ = $v; } }

    macro_rules! set_rr {
        (bc, $v:ident) => { set_bc!($v) };
        (de, $v:ident) => { set_de!($v) };
        (hl, $v:ident) => { set_hl!($v) };
        (ix, $v:ident) => { set_ix!($v) };
        (iy, $v:ident) => { set_iy!($v) };
        (sp, $v:ident) => { set_sp!($v) }
    }

    // Flag operations

    macro_rules! get_cf {
        () => { (f >> CARRY_SHIFT) & 1 }
    }

    // Naming conventions: flags are in the order szhpnc where p can
    // also be v.  All flags must be named.  If a flag is set to a
    // particular value then that value takes place of the flag's
    // name.  If a flag is preserved its name is replaced by 'F'.  If
    // a flag is set randomly its name is replaces by '_'.
    //
    // The operand size follows 'set'.  For operand size 8, the
    // operands and result are 16-bit; for size 16, they are 32-bit.

    macro_rules! sf8 {
        ($result:ident) => { if $result & 0x80 == 0 { 0 } else { SIGN_FLAG } }
    }

    macro_rules! zf8 {
        ($result:ident) => { if $result & 0xFF == 0 { ZERO_FLAG } else { 0 } }
    }

    macro_rules! cf8 {
        ($result:ident) => { if $result & 0x100 == 0 { 0 } else { CARRY_FLAG } }
    }

    macro_rules! set8_szhv0c {
        ($op1:ident, $op2:ident, $result:ident) => {{
            f &= UNUSED_FLAGS;

            let sf = sf8!($result);
            let zf = zf8!($result);
            let hf = 0;         // FIXME
            let vf = 0;         // FIXME
            let cf = cf8!($result);
            f = sf | zf | hf | vf | cf;
        }};
        ($op1:ident, $op2:ident, $op3:ident, $result:ident) => {{
            f &= UNUSED_FLAGS;

            let sf = sf8!($result);
            let zf = zf8!($result);
            let hf = 0;         // FIXME
            let vf = 0;         // FIXME
            let cf = cf8!($result);
            f = sf | zf | hf | vf | cf;
        }}
    }

    macro_rules! set8_sz1p00 {
        ($op1:ident, $op2:ident, $result:ident) => {{
            f &= UNUSED_FLAGS;

            let sf = sf8!($result);
            let zf = zf8!($result);
            let pf = 0;         // FIXME
            f = sf | zf | HALF_FLAG | pf;
        }};
    }

    macro_rules! set8__z1_0F {
        ($v:ident, $bit:ident, $result:ident) => {{
            f &= CARRY_FLAG | UNUSED_FLAGS;

            let zf = zf8!($result);
            f |= zf | HALF_FLAG;
        }}
    }

    macro_rules! sf16 {
        ($result:ident) => { if $result & 0x8000 == 0 { 0 } else { SIGN_FLAG } }
    }

    macro_rules! zf16 {
        ($result:ident) => { if $result & 0xFFFF == 0 { ZERO_FLAG } else { 0 } }
    }

    macro_rules! cf16 {
        ($result:ident) => { if $result & 0x10000 == 0 { 0 } else { CARRY_FLAG } }
    }

    macro_rules! set16_szhv0c {
        ($op1:ident, $op2:ident, $op3:ident, $result:ident) => {{
            f &= UNUSED_FLAGS;

            let sf = sf16!($result);
            let zf = zf16!($result);
            let hf = 0;         // FIXME - carry from bit 11
            let vf = 0;         // FIXME
            let cf = cf16!($result);
            f = sf | zf | hf | vf | cf;
        }};
    }

    macro_rules! set16_FFhF0c {
        ($op1:ident, $op2:ident, $result:ident) => {{
            f &= SIGN_FLAG | ZERO_FLAG | PARITY_FLAG | UNUSED_FLAGS;

            let hf = 0;         // FIXME - carry from bit 11
            let cf = cf16!($result);
            f |= hf | cf;
        }};
    }

    // Memory operations

    macro_rules! byte {
        () => {{
            let c = mem[pc as usize];
            pc = pc.wrapping_add(1);
            c
        }}
    }
    macro_rules! peek_word {
        () => {{
            let lo = mem[pc as usize] as u16;
            let hi = mem[pc.wrapping_add(1) as usize] as u16;
            (hi << 8) | lo
        }}
    }

    macro_rules! at_hl { () => { mem[hl!() as usize] } }

    macro_rules! at_ixd { ($d:ident) => { mem[ix!().wrapping_add($d as u16) as usize] } }
    macro_rules! at_iyd { ($d:ident) => { mem[iy!().wrapping_add($d as u16) as usize] } }

    // Instruction macros

    macro_rules! adc_a_r {
        ($r:ident) => {{
            let cf = get_cf!();
            let result = (a as usize).wrapping_add($r as usize).wrapping_add(cf as usize);
            set8_szhv0c!(a, $r, cf, result);
            a = result as u8;
        }}
    }

    macro_rules! add_a_r {
        ($r:ident) => {{
            let result = (a as usize).wrapping_add($r as usize);
            set8_szhv0c!(a, $r, result);
            a = result as u8;
        }}
    }

    macro_rules! and_a_r {
        ($r:ident) => {{
            let result = (a as usize) & ($r as usize);
            set8_sz1p00!(a, $r, result);
            a = result as u8;
        }}
    }

    macro_rules! adc_hl_ss {
        ($ss:ident) => {{
            let hlval = hl!();
            let ssval = $ss!();
            let cf = get_cf!();
            let result = (hlval as usize).wrapping_add(ssval as usize).wrapping_add(cf as usize);
            set16_szhv0c!(hlval, ssval, cf, result);
            set_hl!(result);
        }}
    }

    macro_rules! add_rr_ss {
        ($rr:ident, $ss:ident) => {{
            let rrval = $rr!();
            let ssval = $ss!();
            let res = (rrval as usize).wrapping_add(ssval as usize);
            set16_FFhF0c!(rrval, ssval, res);
            let result = res as u16;
            set_rr!($rr, result);
        }}
    }

    macro_rules! bit_b {
        // `$bit` is a constant value 0..7
        ($v:ident, $bit:expr) => {{
            let v = $v as u16;
            let bit = (1 << $bit) as u16;
            let res = v & bit;
            set8__z1_0F!(v, bit, res);
        }}
    }

    macro_rules! swap {
        ($a:expr, $b:expr) => {{
            let tmp = $a;
            $a = $b;
            $b = tmp;
        }}
    }

    z80.stop_reason = StopReason::Illegal;
    loop {
        timeslice -= 1;
        if timeslice == 0 {
            z80.stop_reason = StopReason::Poll;
            break;
        }
        match byte!() {
            0x00 => {}
            0x08 => {
                swap!(a, z80.a_alt);
                swap!(f, z80.f_alt);
            }
            0x09 => { add_rr_ss!(hl, bc); } 
            0x19 => { add_rr_ss!(hl, de); } 
            0x29 => { add_rr_ss!(hl, hl); } 
            0x37 => { a = byte!(); }
            0x39 => { add_rr_ss!(hl, sp); } 
            0x76 => {
                z80.stop_reason = StopReason::Halt;
                break;
            }
            0x80 => { add_a_r!(b); }
            0x81 => { add_a_r!(c); }
            0x82 => { add_a_r!(d); }
            0x83 => { add_a_r!(e); }
            0x84 => { add_a_r!(h); }
            0x85 => { add_a_r!(l); }
            0x86 => { let n = at_hl!(); add_a_r!(n); }
            0x87 => { add_a_r!(a); }
            0x88 => { adc_a_r!(b); }
            0x89 => { adc_a_r!(c); }
            0x8A => { adc_a_r!(d); }
            0x8B => { adc_a_r!(e); }
            0x8C => { adc_a_r!(h); }
            0x8D => { adc_a_r!(l); }
            0x8E => { let n = at_hl!(); adc_a_r!(n); }
            0x8F => { adc_a_r!(a); }
            0xA0 => { and_a_r!(b); }
            0xA1 => { and_a_r!(c); }
            0xA2 => { and_a_r!(d); }
            0xA3 => { and_a_r!(e); }
            0xA4 => { and_a_r!(h); }
            0xA5 => { and_a_r!(l); }
            0xA6 => { let n = at_hl!(); and_a_r!(n); }
            0xA7 => { and_a_r!(a); }
            0xC3 => { pc = peek_word!(); }
            0xC6 => { let n = byte!(); add_a_r!(n); }
            0xCB => {
                match byte!() {
                    0x46 => { let n = at_hl!(); bit_b!(n, 0); }
                    0x4E => { let n = at_hl!(); bit_b!(n, 1); }
                    0x56 => { let n = at_hl!(); bit_b!(n, 2); }
                    0x5E => { let n = at_hl!(); bit_b!(n, 3); }
                    0x66 => { let n = at_hl!(); bit_b!(n, 4); }
                    0x6E => { let n = at_hl!(); bit_b!(n, 5); }
                    0x76 => { let n = at_hl!(); bit_b!(n, 6); }
                    0x7E => { let n = at_hl!(); bit_b!(n, 7); }
                    _ =>    { break; }
                }
            }
            0xCE => { let n = byte!(); adc_a_r!(n); }
            0xD3 => {
                z80.port_addr = byte!();
                z80.stop_reason = StopReason::Out;
                break;
            }
            0xD9 => {
                swap!(b, z80.b_alt);
                swap!(c, z80.c_alt);
                swap!(d, z80.d_alt);
                swap!(e, z80.e_alt);
                swap!(h, z80.h_alt);
                swap!(l, z80.l_alt);
            }
            0xDB => {
                z80.port_addr = byte!();
                z80.stop_reason = StopReason::In;
                break;
            }
            0xDD => {
                macro_rules! op_a_ixd {
                    ($op:ident) => {{
                        let d = byte!();
                        let n = at_ixd!(d);
                        $op!(n);
                    }}
                }

                match byte!() {
                    0x09 => { add_rr_ss!(ix, bc); }
                    0x19 => { add_rr_ss!(ix, de); }
                    0x29 => { add_rr_ss!(ix, ix); }
                    0x39 => { add_rr_ss!(ix, sp); }
                    0x86 => { op_a_ixd!(add_a_r); }
                    0x8E => { op_a_ixd!(adc_a_r); }
                    0xA6 => { op_a_ixd!(and_a_r); }
                    _ =>    { break; }
                }
            }
            0xE6 => { let n = byte!(); and_a_r!(n); }
            0xEB => {
                swap!(d, h);
                swap!(e, l);
            }
            0xED => {
                match byte!() {
                    0x4A => { adc_hl_ss!(bc); }
                    0x5A => { adc_hl_ss!(de); }
                    0x6A => { adc_hl_ss!(hl); }
                    0x7A => { adc_hl_ss!(sp); }
                    _ =>    { break; }
                }
            }
            0xFD => {
                macro_rules! op_a_iyd {
                    ($op:ident) => {{
                        let d = byte!();
                        let n = at_iyd!(d);
                        $op!(n);
                    }}
                }

                match byte!() {
                    0x09 => { add_rr_ss!(iy, bc); }
                    0x19 => { add_rr_ss!(iy, de); }
                    0x29 => { add_rr_ss!(iy, iy); }
                    0x39 => { add_rr_ss!(iy, sp); }
                    0x86 => { op_a_iyd!(add_a_r); }
                    0x8E => { op_a_iyd!(adc_a_r); }
                    0xA6 => { op_a_iyd!(and_a_r); }
                    _ =>    { break; }
                }
            }
            _ => { break; }
        }
    }

    z80.pc = pc;
    z80.sp = sp_;
    z80.ix = ix_;
    z80.iy = iy_;
    z80.a = a;
    z80.f = f;
    z80.b = b;
    z80.c = c;
    z80.d = d;
    z80.e = e;
    z80.h = h;
    z80.l = l;
}
