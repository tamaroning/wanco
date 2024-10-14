mod compile;
mod context;
mod driver;
mod inkwell;

use clap::Parser;
use driver::check_config;

fn main() {
    // if RUST_LOG not set, default to info
    /*
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    */

    env_logger::builder().init();
    let args = driver::Args::parse();
    if !check_config(&args) {
        std::process::exit(1);
    }

    if let Err(e) = driver::run_compiler(&args) {
        log::error!("{}", e);
        std::process::exit(1);
    }
}
