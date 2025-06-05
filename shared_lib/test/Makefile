
libpixelsortery_swaylock.so: ../src/lib.rs
	cargo build --lib --release

ctest.out: main.c libpixelsortery_swaylock.so
	gcc main.c -o $@ -L. -lpixelsortery_swaylock

.PHONY: ctest.out

run: ctest.out
	./ctest.out x y



