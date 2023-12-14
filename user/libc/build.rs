use std::{env, path::PathBuf};

use bindgen::MacroTypeVariation;

fn main() {
	let bindings = bindgen::Builder::default()
		.use_core()
		.header("wrapper.h")
		.blocklist_type("__dirstream")
		.default_macro_constant_type(MacroTypeVariation::Signed)
		.clang_arg("-I../../base/usr/include")
		.parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
		.generate()
		.expect("Failed to generate libc bindings.");
	let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
	bindings
		.write_to_file(out_path.join("bindings.rs"))
		.expect("Failed to write libc bindings.");
}
