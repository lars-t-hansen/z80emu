/* Create a boot ROM that loads the first sector of the disk to 100h and jumps
 * to it.
 */

#include <stdio.h>
#include <string.h>

#include "z80.h"

int main(int argc, char** argv) {
    Z80_VARS;

    Z80_RESET_SEC();

    // Set disk parameters and transfer address
    LDA(0);
    OUTA(A_SET_HEAD);
    OUTA(A_SET_TRACK);
    OUTA(A_SET_SECTOR);
    OUTA(A_SET_DMA_LOW);
    LDA(1);
    OUTA(A_SET_DMA_HIGH);

    // Seek
    LDA(A_DISK_OP_CLEAR);
    OUTA(A_DISK_OP);
    LDA(A_DISK_OP_SEEK);
    OUTA(A_DISK_OP);
    // TODO: Technically wait here until status is Done

    // Read
    LDA(A_DISK_OP_CLEAR);
    OUTA(A_DISK_OP);
    LDA(A_DISK_OP_READ);
    OUTA(A_DISK_OP);
    // TODO: Technically wait here until status is Done

    // Invoke loaded program
    JP(0x100);

    FILE *fp = fopen("rom.bin", "w");
    fwrite(Z80_SEC, 1, SECSIZE, fp);
    fclose(fp);
    return 0;
}
