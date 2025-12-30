use crate::aws::error::Result;
use std::io::Write;
use tokio::io::AsyncBufReadExt;
use tokio::signal;
use tokio::sync::mpsc;

pub struct Repl {
    prompt: String, // prompt chars
    line_buf: String,
    input_buf: Vec<String>, // buffer to accumulate stdin input
    // holds complete command for processing
    multiline: bool, // state management of the input
}

impl Repl {
    pub fn new(profile: &str) -> Self {
        Repl {
            prompt: format!("{}> ", profile),
            line_buf: String::new(),
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
Type 'exit;' to quit
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

        self.multiline = false;

        // Begin REPL loop
        loop {
            // self.multiline is a state management flag which is true when input buffer is not empty and not terminated with a semicolon.
            // If the buffer is processing multi-line input, change the prompt into "|"
            if self.multiline {
                print!("| ");
            } else {
                print!("{}", self.prompt);
            }

            // Flush the output to ensure the prompt is displayed immediately
            let _ = std::io::stdout().flush();

            // Wait for either stdin or sigint
            tokio::select! {
                _ = signal::ctrl_c() => {
                    if self.multiline {
                        println!("\n");
                        self.multiline = false;
                        self.line_buf.clear();
                    } else { // Ctrl-C during normal prompt
                        println!("\n(Use Ctrl-D to exit)");
                    }
                }
                Some(line) = rx.recv() => {
                    if !self.multiline {
                        if line.trim().is_empty() { // handle case where user just press Enter (empty input)
                            continue;
                        }
                        if line.trim_end().ends_with(';') {
                            let command = line.trim();
                            if command.to_lowercase() == "exit;" {
                                println!("Exiting Athena CLI!");
                                return Ok(());
                            }
                            println!("Input command: {}", command); // placeholder for actual work
                        } else {
                            self.multiline = true;
                            self.input_buf.push(line);
                        }
                    } else {
                        self.line_buf.push_str(&line);
                        if line.trim_end().ends_with(';') {
                            let command = self
                                .input_buf
                                .iter()
                                .map(|s| s.replace('\n', "").trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect::<Vec<String>>()
                                .join(" ")
                                .replace(" ;", ";");
                            if command.to_lowercase() == "exit;" {
                                println!("Exiting Athena CLI!");
                                return Ok(());
                            }
                            println!("Multiline command: {}", command); // placeholder for string join
                            self.multiline = false;
                            self.input_buf.clear();
                        }
                    }
                }
            }
        }
    }
}
