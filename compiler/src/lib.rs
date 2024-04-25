extern crate proc_macro;

pub use proc_macro::TokenStream;

pub fn handle_tokens(_tokens: TokenStream) -> TokenStream {
    todo!()
}

pub fn dump_ast(path_name: &str, tokens: &proc_macro2::TokenStream) {
    use std::{fs::OpenOptions, io::Write, path::Path, process::Command};
    let path = Path::new(path_name);
    
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .expect(&format!("failed to open `{path_name}`"));
    writeln!(&mut file, "{}", tokens).expect(&format!("failed to write to `{path_name}`"));

    let mut rustfmt = Command::new("rustfmt")
        .arg(path)
        .spawn()
        .expect("failed to spawn `rustfmt`");
    rustfmt.wait().expect("`rustfmt` failed");
}
