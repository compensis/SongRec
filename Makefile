matrix-display=matrix-display/build/librgbmatrix.a matrix-display/rpi-rgb-led-matrix/librgbmatrix.a

all: $(matrix-display)
	cargo build --release --no-default-features

run: $(matrix-display)
	cargo run --release --no-default-features

$(matrix-display):
	$(MAKE) -C matrix-display

clean:
	$(MAKE) -C matrix-display clean