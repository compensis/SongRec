fn main() {
    println!("cargo:rustc-link-search=native=matrix-display/build");
    println!("cargo:rustc-link-lib=static=matrixdisplay");
    println!("cargo:rustc-link-search=native=matrix-display/rpi-rgb-led-matrix/lib");
    println!("cargo:rustc-link-lib=static=rgbmatrix");
}