mod aws;
mod meta;
mod repl;

use inquire::{Select, Text};
use std::error::Error;

use crate::aws::config::{self, build_config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let credential_file_path = aws::config::get_credentials_path()?;

    // Load AWS profile and build Service Config
    let avail_profile = config::get_aws_profile(&credential_file_path)?;
    let selected_profile = Select::new("AWS profile name to use:", avail_profile).prompt()?;
    let input_timeout = Text::new("Put timeout value:").prompt()?;

    println!(
        "\nUsing profile: {} to build SDK config...",
        selected_profile
    );

    let timeout = str::parse::<u64>(input_timeout.as_str())?;
    let no_stall_protection = true;
    let service_config = build_config(&selected_profile, timeout, no_stall_protection).await?;

    // Run the REPL
    let mut repl = repl::Repl::new(&selected_profile);
    repl.repl_loop(service_config).await?;

    // Force Tokio runtime termination to return immediately to OS shell
    std::process::exit(0);
}
