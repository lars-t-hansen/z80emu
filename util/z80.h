#include <stdlib.h>

#define SECSIZE 128

#define Z80_VARS(NUMSEC)                  \
    unsigned char sec_[NUMSEC * SECSIZE]; \
    unsigned lim_ = NUMSEC * SECSIZE;     \
    unsigned tmp_;                        \
    unsigned k_

#define Z80_RESET_SEC() \
    memset(sec_, 0, sizeof(sec_)); \
    k_ = 0

#define Z80_SEC sec_

// TTY
#define CON_OUT 0x00

// Drive A output ports
#define A_SET_HEAD 0x10
#define A_SET_TRACK 0x11
#define A_SET_SECTOR 0x12
#define A_SET_DMA_LOW 0x13
#define A_SET_DMA_HIGH 0x14
#define A_DISK_OP 0x15

/// Disk commands for drive A DISK_OP output port
#define A_DISK_OP_SEEK 0x00
#define A_DISK_OP_READ 0x01
#define A_DISK_OP_WRITE 0x02
#define A_DISK_OP_CLEAR 0x03

// Drive A input ports
#define A_DISK_STATUS 0x10

// Result codes from the disk
#define A_DISK_OK 0x00
#define A_DISK_READY 0x01
// error codes are negative

#define PUT_(n) if (k_ == lim_) abort(); sec_[k_++] = (n)

#define LDA(n)  PUT_(0x37); PUT_(n)
#define OUTA(n) PUT_(0xD3); PUT_(n)
#define JP(n)   tmp_ = (n); PUT_(0xC3); PUT_(tmp_ & 255); PUT_(tmp_ >> 8)
#define HLT()   PUT_(0x76)


