CC = g++
LD = g++
CFLAGS = -Wall -g
LDFLAGS = -lm

all: lib.o wrt.o

lib.o: lib.cc
	$(CC) $(CFLAGS) -c lib.cc -o lib.o $(LDFLAGS)

wrt.o: wrt.cc
	$(CC) $(CFLAGS) -c wrt.cc -o wrt.o $(LDFLAGS)

clean:
	rm -f lib.o wrt.o

.PHONY: all clean
