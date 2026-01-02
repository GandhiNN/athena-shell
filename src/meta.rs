#![allow(unused)]

use crate::aws::athena::AthenaService;

#[derive(Debug, PartialEq)]
pub enum MetaCommand {
    Quit,         // "\q"
    Help,         // "\h"
    ListCatalogs, // "\lc"
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
    \h  Show this help message
    \q  Exit the shell
    \lc List available catalogs

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
    }
    Ok(())
}
