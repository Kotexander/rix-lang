use crate::parser::ParserError;

pub mod ast;
pub mod lexer;
pub mod parser;

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
    pub fn print(&self, error: &ParserError) {
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

fn main() {
    let test_file = "test.rix";
    let test_data = std::fs::read_to_string(test_file).unwrap();

    let mut lexer = lexer::Lexer::new(&test_data);
    loop {
        let token = lexer.next();
        println!("{:?}", token);

        if token.kind.is_eof() {
            break;
        }
    }

    let mut parser = parser::Parser::new(&test_data);
    parser.parse();

    for stmt in parser.ast.stmts() {
        print_stmt(stmt.clone(), &parser);
    }

    let error_printer = ErrorPrinter::new(test_file, &test_data);
    for error in &parser.errors {
        error_printer.print(error);
    }
}

fn print_stmt(stmt: ast::stmt::Stmt, parser: &parser::Parser) {
    match stmt {
        ast::stmt::Stmt::Expr(expr_id) => {
            println!("expression:");
            println!("{}", parser.ast.display_expr(expr_id))
        }
        ast::stmt::Stmt::VarDecl { name, value, .. } => {
            let var_name = parser.ast.resolve_symbol(name);
            println!("var {}:", var_name);
            println!("{}", parser.ast.display_expr(value));
        }
    }
}
