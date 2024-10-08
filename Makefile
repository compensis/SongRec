all: clear_terminal matrixdisplay songrec

clear_terminal:
	clear

matrixdisplay:
	$(MAKE) -C matrix-display

songrec:
	cargo build --release --no-default-features --features matrix-display

clean:
	$(MAKE) -C matrix-display clean
	cargo clean

run: all
	sudo ./target/release/songrec

restart:
	sudo systemctl restart songrec.service
