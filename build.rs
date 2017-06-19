extern crate gcc;

fn main() {
    gcc::Config::new()
        .cpp(true) // Switch to C++ library compilation.
        .file("src/rtimulib_wrapper.cc")
        .compile("librtimulib_wrapper.a");
    println!("cargo:rustc-link-lib=RTIMULib");
}
