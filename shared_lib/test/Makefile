
libpixelsortery.so: src/lib.rs
	cargo build --lib --release

ctest.out: main.c libpixelsortery.so
	gcc main.c -o test/$@ -L. -lpixelsortery

.PHONY: ctest.out

run: ctest.out
	./ctest.out x y



