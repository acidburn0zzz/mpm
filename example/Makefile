INSTALL	= /usr/bin/env install
AR	= /usr/bin/ar
DEST	=
PREFIX	= /usr
BINDIR	= $(PREFIX)/bin
LIBDIR	= $(PREFIX)/lib
MANDIR	= $(PREFIX)/share/man
TRGT	= hello-mpm 
LINK	= gcc 
COMP	= gcc 
CFLAGS	= 
LFLAGS	= 
LIBS	= 
INCPATH	= -I/usr/include 
LIBDIR	= -L/usr/lib 
CLN	= 
OBJ	= test.o
DEL	= rm -f

.SUFFIXES: .o .c .cpp .cc .cxx .C

.cpp.o:
	$(COMP) -c $(CFLAGS) $(INCPATH) -o "$@" "$<"

.cc.o:
	$(COMP) -c $(CFLAGS) $(INCPATH) -o "$@" "$<"

.cxx.o:
	$(COMP) -c $(CFLAGS) $(INCPATH) -o "$@" "$<"

.C.o:
	$(COMP) -c $(CFLAGS) $(INCPATH) -o "$@" "$<"

.c.o:
	$(COMP) -c $(CFLAGS) $(INCPATH) -o "$@" "$<"

all: $(TRGT)

$(TRGT): $(OBJ)
	$(COMP) $(LFLAGS) -o $(TRGT) $(OBJ) $(LIBDIR) $(LIBS)

test.o: test.c
	$(COMP) -c $(CFLAGS) $(INCPATH) -o test.o test.c

clean:
	$(DEL) $(OBJ)
	$(DEL) $(CLN)
	$(DEL) $(TRGT)
	$(DEL) hello-mpm.pkg.tar
	$(DEL) -r build

install:
	$(INSTALL) -d $(DEST)$(BINDIR)
	$(INSTALL) -m755 hello-mpm $(DEST)$(BINDIR)/hello-mpm

uninstall:
	rm -f $(DEST)$(BINDIR)/hello-mpm
