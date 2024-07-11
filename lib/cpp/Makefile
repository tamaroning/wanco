CC = g++
LD = g++
CFLAGS = -Wall -g
LDFLAGS = -lm

all: libwanco.a

libwanco.a: lib.o wrt.o chkpt_decode.o chkpt_encode.o
	ar rcs libwanco.a \
		lib.o \
		wrt.o \
		chkpt_decode.o \
		chkpt_encode.o

lib.o: lib.cc exec_env.h
	$(CC) $(CFLAGS) -c lib.cc -o lib.o $(LDFLAGS)

wrt.o: wrt.cc exec_env.h chkpt.h
	$(CC) $(CFLAGS) -c wrt.cc -o wrt.o $(LDFLAGS)

chkpt_decode.o: chkpt_decode.cc exec_env.h chkpt.h
	$(CC) $(CFLAGS) -c chkpt_decode.cc -o chkpt_decode.o $(LDFLAGS)

chkpt_encode.o: chkpt_encode.cc exec_env.h chkpt.h
	$(CC) $(CFLAGS) -c chkpt_encode.cc -o chkpt_encode.o $(LDFLAGS)

clean:
	rm -f lib.o wrt.o chkpt_decode.o chkpt_encode.o libwanco.a

.PHONY: all clean
