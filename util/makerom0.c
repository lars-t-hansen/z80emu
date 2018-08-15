/* Create a boot ROM that prints a string and halts */

#include <stdio.h>
#include <string.h>

#include "z80.h"

int main(int argc, char** argv) {
    Z80_VARS;

    char k = 0;
    char *text = "Hello, world!\n";

    Z80_RESET_SEC();
    while (*text) {
        LDA(*text++);
        OUTA(CON_OUT);
    }
    HLT();

    FILE *fp = fopen("rom.bin", "w");
    fwrite(Z80_SEC, 1, SECSIZE, fp);
    fclose(fp);
    return 0;
}