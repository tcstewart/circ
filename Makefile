RUSTC ?= rustc

CIRC_LIB := $(shell $(RUSTC) --crate-file-name lib/circ_comms.rs --crate-type=rlib)

all: circ circd $(CIRC_LIB)


circ: $(CIRC_LIB) client/*.rs
	$(RUSTC) client/circ.rs -L.

$(CIRC_LIB): lib/*.rs
	$(RUSTC) lib/circ_comms.rs --crate-type=rlib

circd: $(CIRC_LIB) server/*.rs
	$(RUSTC) server/circd.rs -L.


clean:
	rm -f  circ circd $(CIRC_LIB)

.PHONY: all clean

.SUFFIXES:
