// build.rs

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
  // the compiler tells us which directory to write generated code to
	let _out_dir = std::env::var_os("OUT_DIR").unwrap();


}
