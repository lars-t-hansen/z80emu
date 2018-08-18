/* Create a boot ROM that loads the first sector of the disk to 100h and jumps
 * to it.
 */

#include <stdio.h>
#include <string.h>

#include "z80.h"

int main(int argc, char** argv)
{
    Z80_VARS(1);

    const char* text = "Bleep firmware v0.1\n\n";

    Z80_RESET_SEC();

    // Banner
    while (*text) {
        LDA     (*text++);
        OUTA    (CON_OUT);
    }

    // Set disk parameters and transfer address
    LDA     (0);
    OUTA    (A_SET_HEAD);
    OUTA    (A_SET_TRACK);
    OUTA    (A_SET_SECTOR);
    OUTA    (A_SET_DMA_LOW);
    LDA     (1);
    OUTA    (A_SET_DMA_HIGH);

    // Seek
    LDA     (A_DISK_OP_CLEAR);
    OUTA    (A_DISK_OP);
    LDA     (A_DISK_OP_SEEK);
    OUTA    (A_DISK_OP);
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
    LDA     (A_DISK_OP_CLEAR);
    OUTA    (A_DISK_OP);
    LDA     (A_DISK_OP_READ);
    OUTA    (A_DISK_OP);
    // TODO: Technically wait here until status is Done

    // Invoke loaded program
    JP      (0x100);

    FILE *fp = fopen("rom.bin", "w");
    fwrite(Z80_SEC, 1, SECSIZE, fp);
    fclose(fp);
    return 0;
}
