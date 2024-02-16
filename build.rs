extern crate lalrpop;

fn main() {
    lalrpop::Configuration::new()
        .always_use_colors()
        .log_verbose()
        .process_current_dir()
        .unwrap();
}
