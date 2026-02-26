use std::process;

fn main() {
    if let Err(err) = greentic_i18n_translator::cli::run() {
        eprintln!("{err}");
        process::exit(1);
    }
}
