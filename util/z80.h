#define SECSIZE 128

#define Z80_VARS \
    unsigned char sec_[SECSIZE]; \
    unsigned tmp_;               \
    unsigned k_

#define Z80_RESET_SEC() \
    memset(sec_, 0, sizeof(sec_)); \
    k_ = 0

#define Z80_SEC sec_

// TTY
#define CON_OUT 0x00

// Drive A
#define A_SET_HEAD 0x10
#define A_SET_TRACK 0x11
#define A_SET_SECTOR 0x12
#define A_SET_DMA_LOW 0x13
#define A_SET_DMA_HIGH 0x14
#define A_DISK_OP 0x15

/* Disk commands for DISK_OP */
#define A_DISK_OP_SEEK 0x00
#define A_DISK_OP_READ 0x01
#define A_DISK_OP_WRITE 0x02

#define LDA(n)  sec_[k_++] = 0x37; sec_[k_++] = (n)
#define OUTA(n) sec_[k_++] = 0xD3; sec_[k_++] = (n)
#define JP(n)   tmp_ = (n); sec_[k_++] = 0xC3; sec_[k_++] = tmp_ & 255; sec_[k_++] = tmp_ >> 8
#define HLT()   sec_[k_++] = 0x76

