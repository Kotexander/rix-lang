use crate::lexer::Span;

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub span: Span,
}

pub struct Errors {
    errors: Vec<Error>,
}
impl Errors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add(&mut self, message: impl Into<String>, span: Span) {
        if let Some(last) = self.errors.last()
            && last.span == span
        {
            // don't add duplicate errors for the same span
            return;
        }
        self.errors.push(Error {
            message: message.into(),
            span,
        });
    }
    pub fn sort(&mut self) {
        self.errors.sort_by_key(|e| e.span);
    }
}
impl<'a> IntoIterator for &'a Errors {
    type Item = &'a Error;
    type IntoIter = std::slice::Iter<'a, Error>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.iter()
    }
}

pub struct ErrorPrinter<'input> {
    file: &'input str,
    input: &'input str,
    lines: Vec<u32>,
}
impl<'input> ErrorPrinter<'input> {
    pub fn new(file: &'input str, input: &'input str) -> Self {
        assert!(input.len() < u32::MAX as usize);

        let mut acc = 0;
        let lines = input
            .split_inclusive('\n')
            .map(|line| {
                let len = acc;
                acc += line.len() as u32;
                len
            })
            .collect();

        Self { file, input, lines }
    }
    fn get_line_col(&self, pos: u32) -> (u32, u32) {
        let mut line_num = 0;
        let mut line_start = 0;
        for (i, &start) in self.lines.iter().enumerate() {
            if start > pos {
                break;
            }
            line_num = i as u32;
            line_start = start;
        }
        let col_num = pos - line_start + 1;
        (line_num + 1, col_num)
    }
    fn get_line_str(&self, line: u32) -> &str {
        let start = self.lines[line as usize - 1] as usize;
        let end = if (line as usize) < self.lines.len() {
            self.lines[line as usize] as usize
        } else {
            self.input.len()
        };
        self.input[start..end].trim_ascii_end()
    }
    pub fn print(&self, error: &Error) {
        let (line, col) = self.get_line_col(error.span.start);
        eprintln!(
            "\x1b[1m\x1b[31merror\x1b[0m\x1b[1m: {}\x1b[0m",
            error.message
        );
        eprintln!("\x1b[36m -->\x1b[0m {}:{}:{}", self.file, line, col);
        // eprintln!("\x1b[36m{} |\x1b[0m {}", line, &self.input[error.span])
        eprintln!("\x1b[36m  |\x1b[0m");
        eprintln!("\x1b[36m{} |\x1b[0m {}", line, &self.get_line_str(line));
        eprintln!("\x1b[36m  |\x1b[0m");
    }
}
