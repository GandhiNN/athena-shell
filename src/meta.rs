#![allow(unused)]
#[derive(Debug, PartialEq)]
pub enum MetaCommand {
    Quit,          // "\q"
    Help,          // "\h"
    ListDatabases, // "\l"
}

pub fn execute_meta_command(cmd: MetaCommand) -> Result<(), Box<dyn std::error::Error>> {
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
    \l  List available databases

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
        MetaCommand::ListDatabases => {
            println!("Listing Databases")
        }
    }
    Ok(())
}
