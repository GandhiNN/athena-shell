use crate::aws::error::Result;

use crate::meta::{MetaCommand, execute_meta_command};
use std::io::Write;
use tokio::io::AsyncBufReadExt;
use tokio::signal;
use tokio::sync::mpsc;

pub struct Repl {
    prompt: String,         // prompt chars
    input_buf: Vec<String>, // buffer to accumulate stdin input
    multiline: bool,        // state management of the input
}

impl Repl {
    pub fn new(profile: &str) -> Self {
        Repl {
            prompt: format!("{}> ", profile),
            input_buf: Vec::new(),
            multiline: false,
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
Type '\h' to show help
Type '\q' to exit from shell
        "#
        )
    }

    pub async fn repl_loop(&mut self) -> Result<()> {
        // Print header when first time entering the shell
        self.print_header();

        // Create a channel for delivering lines from stdin task
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();

        // Create a dedicated stdin reader task
        tokio::spawn(async move {
            let stdin = tokio::io::stdin();
            let mut reader = tokio::io::BufReader::new(stdin);
            let mut line = String::new();

            // Check for stdin input from keyboard
            // SIGNAL is not treated as a valid stdin input from keyboard
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF -> 0 bytes read
                    Ok(_) => {
                        if tx.send(line.clone()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // By default the shell is not in multiline mode
        self.multiline = false;

        // Begin REPL loop
        loop {
            // If the buffer is processing multi-line input, change the prompt into "|"
            if self.multiline {
                print!("| ");
            } else {
                print!("{}", self.prompt);
            }

            // Flush the output to ensure the prompt is displayed immediately
            let _ = std::io::stdout().flush();

            // Wait for either SIGINT or STDIN
            tokio::select! {
                _ = signal::ctrl_c() => {
                    if self.multiline {
                        self.multiline = false;
                        println!("\n");
                        self.input_buf.clear();
                    } else { // Ctrl-C during normal prompt
                        println!("\n(Use Ctrl-D to exit)");
                    }
                }
                result = rx.recv() => {
                    match result {
                        Some(line) => {
                            if !self.multiline {
                                if line.trim().is_empty() { // handle case where user just press Enter (empty input)
                                    continue;
                                }
                                match line.trim() {
                                    "\\h" => {
                                        let meta = MetaCommand::Help;
                                        let _ = execute_meta_command(meta);
                                        continue
                                    }
                                    "\\q" => {
                                        let meta = MetaCommand::Quit;
                                        let _ = execute_meta_command(meta);
                                        return Ok(());
                                    }
                                    _ => {
                                        if line.trim_end().ends_with(';') {
                                            let command = String::from(line.trim());
                                            println!("Input command: {}", command); // placeholder for actual work
                                        } else {
                                            self.multiline = true;
                                            self.input_buf.push(line);
                                        }
                                    }
                                }
                            } else {
                                self.input_buf.push(line.clone());
                                if line.trim_end().ends_with(';') {
                                    let command = self
                                        .input_buf
                                        .iter()
                                        .map(|s| s.replace('\n', "").trim().to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect::<Vec<String>>()
                                        .join(" ")
                                        .replace(" ;", ";");

                                    println!("Multiline command: {}", command);
                                    self.multiline = false;
                                    self.input_buf.clear();
                                }
                            }
                        }
                        None => {
                            // Handle EOF (Ctrl-D)
                            println!("\nExiting Athena CLI!");
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}
