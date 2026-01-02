mod aws;
mod meta;
mod repl;

use inquire::Text;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let res = aws::config::get_credentials_path().unwrap();
    println!("{}", res.display());

    // Load AWS profile and build Athena client
    let profile: String = loop {
        let input = Text::new("AWS profile name to use:").prompt()?;
        if !input.is_empty() {
            break input;
        }
    };

    // Run the REPL
    let mut repl = repl::Repl::new(&profile);
    repl.repl_loop().await?;

    // Force Tokio runtime termination to return immediately to OS shell
    std::process::exit(0);
}
