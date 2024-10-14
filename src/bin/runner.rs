use std::env;
use std::error::Error;
use std::path::Path;
use log::{error, info};
use alfred_rs::config::Config;
use std::process::Command;

#[cfg(feature = "runner")]
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let config = Config::read(Some("runner".into()))?;
    let modules = if args.len() == 1 {
        println!("Args is empty, loading from config.toml...");
        let alfred_config = Config::read(None)?.alfred;
        println!("{:?}", config);
        alfred_config.modules
    } else {
        println!("args is not empty: {:?}", args);
        args
    };
    println!("{:?}", modules);
    let rust_log_env = if let Some(log) = config.get_module_value("log".into()) {
        if matches!(log.as_str(), "debug" | "info" | "warn" | "error") {
            log
        } else {
            "".into()
        }
    } else {
        "".into()
    };

    modules.iter().for_each(|module| {
        // check if file exists
        if !Path::new(&format!("./{module}")).exists() {
            error!("module {module} not found");
        } else {
            info!("running module '{}'...", module);
            Command::new("sh")
                .arg("-c")
                .env("RUST_LOG", rust_log_env.clone())
                .arg(format!("./{module}"))
                .spawn()
                .expect("failed to execute process");
        }
    });
    Ok(())
}