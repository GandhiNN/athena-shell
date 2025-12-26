#![allow(unused)]
use crate::error::Result;
use aws_sdk_athena::Client as AthenaClient;
use ctrlc;
use std::io;
use std::io::Write;
use std::sync::mpsc;
use std::thread;

pub struct Repl {
    prompt: String,        // prompt chars
    input_buf: String,     // buffer to accumulate stdin input
    line_buf: String,      // holds complete command for processing
    is_in_multiline: bool, // state management of the input
}

impl Repl {
    pub fn new(profile: &str) -> Self {
        Repl {
            prompt: String::from(profile),
            input_buf: String::new(),
            line_buf: String::new(),
            is_in_multiline: false,
        }
    }

    pub fn print_header(&self) {
        println!(
            r#"
╔═══════════════════════════════════════╗
║           ATHENA SHELL                ║
║     AWS Query Interface v0.1.0        ║
╚═══════════════════════════════════════╝

AWS Athena Query Interface - v0.1.0
Type 'exit;' to quit
        "#
        )
    }

    pub async fn repl_loop(&mut self) -> Result<()> {
        // Print header when first time entering the shell
        self.print_header();
        // Begin REPL loop
        loop {
            // Read line from stdin and flush immediately to stdout.
            // is_in_multiline is a state management flag which is true when input buffer is not empty and not terminated with a semicolon.
            // In this case it creates a new line feed prefixed with a pipe "| " char.
            // If input buffer is terminated with a semicolon, then close the input stream and process the command.
            // If return char (\n or \r\n) is fed to the buffer, then clear the input buffer.
            //
            // If the buffer is processing multi-line input, change the prompt into "|"
            if self.is_in_multiline {
                print!("| ");
            } else {
                print!("{}", self.prompt);
            }

            // Flush the output to ensure the prompt is displayed immediately
            io::stdout().flush().unwrap();

            // Read line from stdin into the input buffer
            io::stdin().read_line(&mut self.input_buf).unwrap();

            if self.input_buf.contains(";") {
                // Command complete, move and process
                self.line_buf = std::mem::take(&mut self.input_buf);
                self.is_in_multiline = false;
                // Sanitize the line_buf string logic
                let command = self
                    .line_buf
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .join(" ")
                    .replace(" ;", ";");
                if command == "exit;" {
                    println!("Exiting Athena Shell!");
                    return Ok(());
                }
                // Execute command
                println!("{}", command); // placeholder for command execution
                // Empty the line buffer
                // input buffer should already empty after the take()
                self.line_buf.clear();
            } else {
                self.is_in_multiline = true
            }
        }
    }
}
