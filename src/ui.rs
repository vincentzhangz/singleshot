use std::io::Write;

const BOX_MIN_WIDTH: usize = 40;

pub struct BoxPrinter {
    width: usize,
}

impl BoxPrinter {
    pub fn new(content_width: usize) -> Self {
        Self {
            width: content_width.max(BOX_MIN_WIDTH) + 4,
        }
    }

    #[allow(dead_code)]
    pub fn from_lines(lines: &[String], title: &str) -> Self {
        let max_width = lines
            .iter()
            .map(|l| l.len())
            .max()
            .unwrap_or(BOX_MIN_WIDTH)
            .max(title.len());
        Self::new(max_width)
    }

    pub fn print_top(&self) {
        eprintln!("╭{}╮", "─".repeat(self.width - 2));
    }

    pub fn print_line(&self, text: &str) {
        eprintln!("│ {:<width$} │", text, width = self.width - 4);
    }

    pub fn print_separator(&self) {
        eprintln!("├{}┤", "─".repeat(self.width - 2));
    }

    pub fn print_bottom(&self) {
        eprintln!("╰{}╯", "─".repeat(self.width - 2));
    }

    #[allow(dead_code)]
    pub fn print_box(&self, title: &str, lines: &[String]) {
        self.print_top();
        self.print_line(title);
        self.print_separator();
        for line in lines {
            self.print_line(line);
        }
        self.print_bottom();
    }
}

pub fn status(message: &str) {
    eprint!("[*] {}", message);
    std::io::stderr().flush().ok();
}

pub fn status_done(message: &str) {
    eprintln!("\r[+] {}", message);
}

pub fn status_done_detail(message: &str, detail: &str) {
    eprintln!("\r[+] {} ({})", message, detail);
}

pub fn newline() {
    eprintln!();
}

pub fn completed(message: &str) {
    eprintln!("[+] {}", message);
}
