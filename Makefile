install_prereqs:
	sudo apt-get install build-essential -y
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
	cargo install bootimage
	cargo install cargo-release
	rustup component add rust-src
	rustup component add llvm-tools-preview
	sudo apt-get install qemu-system -y


clean:
	cargo clean
	tar -cf drive.img root
	

build:

	cargo build

run:
	tar -cf drive.img root
	cargo run --release

debug-bp:
	cargo run --features log_debug,breakpoints

debug:
	cargo run --features log_debug

run_headless:
	cargo build
	cargo bootimage
	qemu-system-x86_64 -drive format=raw,file=target/x86-64_cobalt/debug/bootimage-kernel.bin -display none -serial stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04 -m 2m

test_boot:
	qemu-system-x86_64 -hda drive.img -serial stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04 -m 32m


test:
	cargo test

patch:
	git add .
	git commit --amend -m "Release Patch"
	cargo release patch

release:
	git add .
	git commit --amend -m "Release"
	cargo release
