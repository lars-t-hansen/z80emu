extern crate z80asm;

use z80asm::*;

const BANNER: &str = "Bleep firmware v0.1\n\n";
const LOADADDR: u16 = 0x100;

fn main()
{
    let mut z = Z80Buf::new(1);

    // Banner
    for c in BANNER.as_bytes() {
        z.lda(*c);
        z.outa(CON_OUT)
    }

    // Set disk parameters and transfer address
    z.lda(0);
    z.outa(A_SET_HEAD);
    z.outa(A_SET_TRACK);
    z.outa(A_SET_SECTOR);
    z.outa(A_SET_DMA_LOW);
    assert_eq!(LOADADDR & 0xFF, 0);
    z.lda((LOADADDR >> 8) as u8);
    z.outa(A_SET_DMA_HIGH);

    // Seek
    z.lda(A_DISK_OP_CLEAR);
    z.outa(A_DISK_OP);
    z.lda(A_DISK_OP_SEEK);
    z.outa(A_DISK_OP);
    
    // Read
    z.lda(A_DISK_OP_CLEAR);
    z.outa(A_DISK_OP);
    z.lda(A_DISK_OP_SEEK);
    z.outa(A_DISK_OP);
    
    // Invoke loaded program
    z.jp(LOADADDR);

    // Write ROM image
    z.create_image("rom.bin");
}
