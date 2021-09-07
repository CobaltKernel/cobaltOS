install_prereqs:
	cargo install bootimage
	rustup component add llvm-tools-preview
	apt-get install qemu-

clean:
	cargo clean

build:
	cargo build

run:
	cargo run

run_headless:
	cargo build
	cargo bootimage
	qemu-system-x86_64 -drive format=raw,file=target/x86-64_cobalt/debug/bootimage-kernel.bin -display none -serial stdio
