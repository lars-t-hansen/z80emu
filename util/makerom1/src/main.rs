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
    // TODO: Technically wait here until status is Done
    
/*
    FWD     (L_seek_ready);
    FWD     (L_io_err);

    LABEL   (L_seek_wait);
    INA     (A_DISK_STATUS);
    CPA     (A_DISK_READY);
    JPZ     (L_seek_ready);
    CPA     (A_DISK_OK);
    JPZ     (L_seek_wait);
    JP      (L_io_err);
    HERE    (L_seek_ready);
*/

    // Read
    z.lda(A_DISK_OP_CLEAR);
    z.outa(A_DISK_OP);
    z.lda(A_DISK_OP_READ);
    z.outa(A_DISK_OP);
    // TODO: Technically wait here until status is Done
    
    // Invoke loaded program
    z.jp(LOADADDR);

    // Write ROM image
    z.create_image("rom.bin");
}
