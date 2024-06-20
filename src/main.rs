mod compile;
mod context;
mod driver;
mod inkwell;

use clap::Parser;

fn main() {
    /*
    // if RUST_LOG not set, default to info
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    */

    env_logger::builder().init();
    let args = driver::Args::parse();

    if let Err(e) = driver::run(&args) {
        log::error!("{}", e);
        std::process::exit(1);
    }
}
