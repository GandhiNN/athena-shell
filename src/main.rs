mod aws;
mod repl;

use inquire::Text;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let res = aws::config::get_credentials_path().unwrap();
    println!("{}", res.display());

    // Load AWS profile and build Athena client
    let mut profile = String::new();
    loop {
        profile = Text::new("AWS profile name to use:").prompt()?;
        if !profile.is_empty() {
            break;
        }
    }
    println!("{}", profile);

    // Run the REPL
    let mut repl = repl::Repl::new(&profile);
    repl.repl_loop().await?;

    Ok(())
}
