mod analysis;
mod ast;
mod errors;
mod lexer;
// mod llvm;
mod parser;
mod tir;

fn main() {
    test();
}

fn test() {
    let test_file = "test.rix";
    let test_data = std::fs::read_to_string(test_file).unwrap();

    let mut errors = errors::Errors::new();
    let mut parser = parser::Parser::new(&test_data, &mut errors);
    parser.parse();

    let ast = parser.finish();
    analysis::analyze(&ast, &mut errors);

    println!("Parsed AST:\n{}", ast);

    let error_printer = errors::ErrorPrinter::new(test_file, &test_data);
    errors.sort();
    for error in &errors {
        error_printer.print(error);
    }
}
