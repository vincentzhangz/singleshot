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
    fn test_box_printer_from_lines_empty() {
        let lines: Vec<String> = vec![];
        let printer = BoxPrinter::from_lines(&lines, "Title");
        assert_eq!(printer.width, BOX_MIN_WIDTH + 4);
    }

    #[test]
    fn test_box_printer_from_lines_uses_max_line_length() {
        let lines = vec![
            "Short".to_string(),
            "This is a much longer line that exceeds minimum".to_string(),
            "Medium length".to_string(),
        ];
        let printer = BoxPrinter::from_lines(&lines, "Title");
        let expected_width = "This is a much longer line that exceeds minimum".len() + 4;
        assert_eq!(printer.width, expected_width);
    }

    #[test]
    fn test_box_printer_from_lines_uses_title_if_longest() {
        let lines = vec!["Short".to_string()];
        let long_title = "This is a very long title that is the longest";
        let printer = BoxPrinter::from_lines(&lines, long_title);
        let expected_width = long_title.len() + 4;
        assert_eq!(printer.width, expected_width);
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

    #[test]
    fn test_box_printer_from_lines_with_empty_lines() {
        let lines = vec!["".to_string(), "Content".to_string(), "".to_string()];
        let printer = BoxPrinter::from_lines(&lines, "T");
        assert_eq!(printer.width, BOX_MIN_WIDTH + 4);
    }
}
