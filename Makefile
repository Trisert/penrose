all:
	cargo update; cargo build --release

install:
	rm -rf ~/.local/bin/penrose ; cp ~/penrose/target/release/penrose ~/.local/bin
