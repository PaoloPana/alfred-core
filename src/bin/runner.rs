use std::env;
use std::path::Path;
use log::{error, info};
use alfred_rs::config::Config;
use std::process::Command;

#[cfg(feature = "runner")]
#[allow(clippy::print_stdout,clippy::use_debug)]
fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::read(Some("runner"));
    let modules = if args.len() == 1 {
        println!("Args is empty, loading from config.toml...");
        let alfred_config = Config::read(None).alfred;
        alfred_config.modules
    } else {
        println!("args is not empty: {args:?}");
        args
    };
    println!("{modules:?}");
    let rust_log_env = config.get_module_value("log")
        .map_or(
            String::new(),
            |log| if matches!(log.as_str(), "debug" | "info" | "warn" | "error") { log } else { String::new() }
        );

    for module in modules {
        // check if file exists
        if Path::new(&format!("./{module}")).exists() {
            info!("running module '{}'...", module);
            Command::new("sh")
                .arg("-c")
                .env("RUST_LOG", rust_log_env.clone())
                .arg(format!("./{module}"))
                .spawn()
                .expect("failed to execute process");
        } else {
            error!("module {module} not found");
        }
    }
}