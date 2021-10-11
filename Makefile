install_prereqs:
	cargo install bootimage
	rustup component add rust-src
	rustup component add llvm-tools-preview
	apt-get install qemu-system


clean:
	cargo clean
	qemu-img create drive.img 64M

build:
	cargo build

run:
	cargo run

debug:
	cargo run --features log_debug,breakpoints

run_headless:
	cargo build
	cargo bootimage
	qemu-system-x86_64 -drive format=raw,file=target/x86-64_cobalt/debug/bootimage-kernel.bin -display none -serial stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04 -m 2m

test_boot:
	qemu-system-x86_64 -hda drive.img -serial stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04 -m 32m
