mod aws;
mod meta;
mod repl;

use inquire::Select;
use std::error::Error;

use crate::aws::athena::AthenaService;
use crate::aws::config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let res = aws::config::get_credentials_path().unwrap();
    println!("{}", res.display());

    // Load AWS profile and build Athena client
    let avail_profile = config::get_aws_profile(&res)?;
    let selected_profile = Select::new("AWS profile name to use:", avail_profile).prompt()?;

    println!(
        "\nUsing profile: {} to build SDK config...",
        selected_profile
    );

    // Build Athena Client
    let timeout = 10000;
    let no_stall_protection = true;
    let athena_service =
        AthenaService::new(selected_profile.as_str(), timeout, no_stall_protection).await?;

    // Loading available data catalogue for the selected profile
    let avail_catalogues = athena_service.list_catalogs().await?;
    let selected_catalogue = Select::new("Athena Catalogue to use:", avail_catalogues).prompt()?;

    // Run the REPL
    let mut repl = repl::Repl::new(&selected_profile, &selected_catalogue);
    repl.repl_loop(athena_service).await?;

    // Force Tokio runtime termination to return immediately to OS shell
    std::process::exit(0);
}
