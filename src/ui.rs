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
}

pub fn status(message: &str) {
    eprint!("[*] {}", message);
    std::io::stderr().flush().ok();
}

pub fn status_done(message: &str) {
    eprintln!("\r[+] {}\x1b[K", message);
}

pub fn status_done_detail(message: &str, detail: &str) {
    eprintln!("\r[+] {} ({})\x1b[K", message, detail);
}

pub fn newline() {
    eprintln!();
}

pub fn completed(message: &str) {
    eprintln!("[+] {}", message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_printer_new_minimum_width() {
        let printer = BoxPrinter::new(10);
        assert_eq!(printer.width, BOX_MIN_WIDTH + 4);
    }

    #[test]
    fn test_box_printer_new_larger_width() {
        let printer = BoxPrinter::new(50);
        assert_eq!(printer.width, 54);
    }

    #[test]
    fn test_box_min_width_constant() {
        assert_eq!(BOX_MIN_WIDTH, 40);
    }

    #[test]
    fn test_box_printer_width_calculation() {
        let printer1 = BoxPrinter::new(0);
        assert_eq!(printer1.width, 44);

        let printer2 = BoxPrinter::new(39);
        assert_eq!(printer2.width, 44);

        let printer3 = BoxPrinter::new(40);
        assert_eq!(printer3.width, 44);

        let printer4 = BoxPrinter::new(41);
        assert_eq!(printer4.width, 45);
    }
}
