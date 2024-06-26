CC = gcc
CFLAGS = -Wall -g
LDFLAGS = -lm

all: lib.o wrt.o

lib.o: lib.c
	$(CC) $(CFLAGS) -c lib.c -o lib.o $(LDFLAGS)

wrt.o: wrt.c
	$(CC) $(CFLAGS) -c wrt.c -o wrt.o $(LDFLAGS)

clean:
	rm -f lib.o wrt.o

.PHONY: all clean
