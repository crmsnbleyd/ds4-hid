use ds4::runner;
fn main() {
    if let Err(e) = runner::run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
