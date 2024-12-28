use std::env;
use std::path::Path;
use log::{error, info};
use alfred_rs::config::Config;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

fn is_keep_alive(args: &[String], config: &Config) -> bool {
    args.iter().any(|a| a == "--keep-alive" || a == "-k")
        || config.get_module_value("keep_alive").map_or_else(|| false, |v| v == "true")
}

#[allow(clippy::use_debug)]
fn main() {
    env_logger::init();
    let mut args: Vec<String> = env::args().collect();
    let config = Config::read(Some("runner"));
    let keep_alive = is_keep_alive(&args, &config);
    let modules = if args.len() == 1 || args.len() == 2 && keep_alive {
        info!("Args is empty, loading from config.toml...");
        &config.alfred.modules
    } else {
        info!("args is not empty: {args:?}");
        &args.drain(1..).into_iter().filter(|arg| !arg.starts_with("--")).collect::<Vec<_>>()
    };
    info!("{modules:?}");
    let rust_log_env = config.get_module_value("log")
        .map_or(
            String::new(),
            |log| if matches!(log.as_str(), "debug" | "info" | "warn" | "error") { log } else { String::new() }
        );

    for module in modules {
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
    if keep_alive {
        loop {
            sleep(Duration::from_secs(10));
        }
    }
}