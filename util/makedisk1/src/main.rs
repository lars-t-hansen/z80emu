extern crate z80asm;

use z80asm::*;

const GREETING: &str = "Hello, world!\n";
const ORG: u16 = 0x100;

fn main()
{
    let mut z = Z80Buf::new(1, ORG);

    for c in GREETING.as_bytes() {
        z.lda(*c);
        z.outa(CON_OUT)
    }
    z.hlt();

    z.create_image("a_drive.bin");
}
