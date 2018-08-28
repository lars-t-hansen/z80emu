.PHONY: all

all: rom.bin a_drive.bin

rom.bin: makerom1/src/main.rs z80asm/src/lib.rs
	( cd makerom1 ; cargo run ; mv rom.bin .. )

a_drive.bin: makedisk1/src/main.rs z80asm/src/lib.rs
	( cd makedisk1 ; cargo run ; mv a_drive.bin .. )
