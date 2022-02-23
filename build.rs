fn main() {
    println!("cargo:rustc-link-search=native=Amazons");
    println!("cargo:rustc-link-lib=static=amazons");
}
