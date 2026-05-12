pub struct RoX {
    pub had_error: bool,
}

impl RoX {
    pub fn new() -> Self {
        RoX { had_error: false }
    }

    pub fn report_error(&mut self, line: u64, message: &str) {
        println!("[line {}] Error: {}", line, message);
        self.had_error = true;
    }
}
