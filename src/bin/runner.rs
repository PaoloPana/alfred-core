use std::env;
use std::error::Error;
use log::info;
use alfred_rs::config::Config;

#[cfg(feature = "runner")]
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let modules = if args.len() == 1 {
        println!("Args is empty, loading from config.toml...");
        let config = Config::read(None)?;
        let alfred_config = Config::read_alfred_config()?;
        println!("{:?}", config);
        alfred_config.get_modules().unwrap()
    } else {
        println!("args is not empty: {:?}", args);
        args
    };
    println!("{:?}", modules);
    Ok(())
}