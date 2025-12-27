#![allow(dead_code)]
mod athena;
mod client;
mod config;
mod error;
mod repl;

use inquire::Text;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let res = config::get_credentials_path().unwrap();
    println!("{}", res.display());

    // Load AWS profile and build Athena client
    let profile: String = Text::new("AWS profile name to use:").prompt()?;
    println!("{}", profile);

    // Run the REPL
    let mut repl = repl::Repl::new(&profile);
    repl.repl_loop().await?;

    Ok(())
}
