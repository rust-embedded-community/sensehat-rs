extern crate gcc;

#[cfg(feature = "rtimu")]
fn main() {
    gcc::Build::new()
        .cpp(true) // Switch to C++ library compilation.
        .file("src/rtimulib_wrapper.cc")
        .compile("librtimulib_wrapper.a");
    println!("cargo:rustc-link-lib=RTIMULib");
}

#[cfg(not(feature = "rtimu"))]
fn main() {}
