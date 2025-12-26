#![allow(unused)]
use crate::error::Result;
use aws_sdk_athena::Client as AthenaClient;
use std::io::Write;
use std::io::{self, BufReader};
use std::sync::mpsc;
use std::thread;
use tokio::io::{AsyncBufReadExt, AsyncReadExt};
use tokio::io::{BufReader as TokioBufReader, Stdin};

pub struct Repl {
    prompt: String, // prompt chars
    line_buf: String,
    input_buf: Vec<String>, // buffer to accumulate stdin input
    // holds complete command for processing
    is_in_multiline: bool, // state management of the input
}

impl Repl {
    pub fn new(profile: &str) -> Self {
        Repl {
            prompt: format!("{}> ", profile),
            line_buf: String::new(),
            input_buf: Vec::new(),
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

    pub async fn read_line(&self) -> Result<String> {
        // Create the reader
        let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
        let mut buffer = Vec::new();
        let _fut = reader.read_until(b'\n', &mut buffer).await;
        Ok(String::from_utf8(buffer)?)
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

            // Wait for either stdin or sigint
            tokio::select! {
                linebuf = self.read_line() => {
                    let tmp = linebuf.unwrap();
                    self.input_buf.push(tmp.clone());
                    if tmp.contains(";") {
                        // Command complete, move and process
                        // self.line_buf = std::mem::take(&mut self.input_buf);
                        self.is_in_multiline = false;
                        // Sanitize the line_buf string logic
                        let command = self
                            .input_buf
                            .iter()
                            .map(|s| s.replace('\n', "").trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect::<Vec<String>>()
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
                        self.input_buf.clear();
                    } else if tmp == "\n" || tmp == "\r\n" { // checks for newline-only input
                        if !self.input_buf.is_empty() {
                            self.is_in_multiline = true; // keep printing "|" as prompt if input_buf contains data
                        } else {
                            self.input_buf.clear(); // regenerate prompt for newline-only input
                            self.is_in_multiline = false;
                        }
                    } else {
                        self.is_in_multiline = true;
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    println!("\nCtrl+C detected, cancelling current input...");
                    println!("Press ENTER twice to return to the prompt");
                    self.input_buf.clear();
                    self.line_buf.clear();
                    self.is_in_multiline = false;
                    // consume pending input
                    let _ = tokio::io::stdin().read(&mut [0u8; 1024]).await;
                    tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                }
            }
        }
    }
}
