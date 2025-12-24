pub struct Repl {
    prompt: String,
    buffer: String,
    previous_lines: Vec<String>,
    is_in_multiline: bool,
}

impl Repl {
    // The REPL constructor.
    pub fn new(prompt: String) -> Self {
        Repl {
            prompt,
            buffer: String::new(),
            previous_lines: Vec::<String>::new(),
            is_in_multiline: false,
        }
    }
}
