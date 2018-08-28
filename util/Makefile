.PHONY: all

PROGRAMS=makerom0 makerom1 makedisk1

all: $(PROGRAMS)

clean:
	rm -f $(PROGRAMS) *.o

*.o: z80.h

makerom0: makerom0.o
makerom1: makerom1.o
makedisk1: makedisk1.o
