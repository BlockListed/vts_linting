use std::path::PathBuf;

fn main() {
    let mut units_out_path: PathBuf = std::env::var_os("OUT_DIR").unwrap().into();
    units_out_path.push("units.rs");

    let tokens = codegen::generate("../units.json");

    let file: syn::File = syn::parse2(tokens).unwrap();

    let output = prettyplease::unparse(&file);

    std::fs::write(units_out_path, output).expect("Couldn't write generated code");
}
