#![allow(unused)]
#[derive(Debug, PartialEq)]
pub enum MetaCommand {
    Quit, // "\q"
    Help, // "\h"
}

pub fn execute_meta_command(cmd: MetaCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        MetaCommand::Help => {
            println!("Placeholder for HELP")
        }
        MetaCommand::Quit => {
            println!("Exiting Shell!")
        }
    }
    Ok(())
}
