
libpixelsortery.so: src/lib.rs
	cargo build --lib --release

ctest.out: libpixelsortery.so main.c cpixelsortery.h
	gcc main.c -o $@ -L. -lpixelsortery

run: ctest.out
	./ctest.out x y
