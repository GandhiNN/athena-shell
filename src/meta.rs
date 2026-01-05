#![allow(unused)]

use crate::aws::athena::AthenaService;

#[derive(Debug, PartialEq)]
pub enum MetaCommand {
    Quit,                  // "\q"
    Help,                  // "\h"
    ListCatalogs,          // "\lc"
    ListDatabases(String), // "\ld <catalog_name> - catalog name as parameter"
}

pub async fn execute_meta_command(
    cmd: MetaCommand,
    service: &AthenaService,
) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        MetaCommand::Help => {
            println!(
                r#"
╔═══════════════════════════════════════╗
║           ATHENA SHELL                ║
║     AWS Query Interface v0.1.0        ║
╚═══════════════════════════════════════╝

Meta Commands:
    \h                 Show this help message
    \q                 Exit the shell
    \lc                List available catalogs
    \ld <catalog_name> List available databases under catalog

Query Commands:
    End statements with semicolon (;) to execute
    Multi-line queries supported

Controls:
    Ctrl-C  Cancel current input / Clear multi-line buffer
    Ctrl-D  Exit shell
"#
            );
        }
        MetaCommand::Quit => {
            println!("Exiting Shell!")
        }
        MetaCommand::ListCatalogs => {
            println!("Listing Catalogues");
            let cat = service.list_catalogs().await?;
            println!("{:?}", cat);
        }
        MetaCommand::ListDatabases(catalog_name) => {
            println!("Listing Databases for catalog: {}", catalog_name);
            let dbs = service.list_databases(&catalog_name).await?;
            println!("{:?}", dbs);
        }
    }
    Ok(())
}
