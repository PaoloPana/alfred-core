use std::env;
use std::error::Error;
use log::info;
use alfred_rs::config::Config;
use std::process::Command;

#[cfg(feature = "runner")]
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let config = Config::read(Some("runner".into()))?;
    let modules = if args.len() == 1 {
        println!("Args is empty, loading from config.toml...");
        let alfred_config = Config::read_alfred_config()?;
        println!("{:?}", config);
        alfred_config.get_modules().unwrap()
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
        // TODO: check if file exists
        info!("running module '{}'...", module);
        Command::new("sh")
            .arg("-c")
            .env("RUST_LOG", rust_log_env.clone())
            .arg(format!("./{module}"))
            .spawn()
            .expect("failed to execute process");
    });
    Ok(())
}