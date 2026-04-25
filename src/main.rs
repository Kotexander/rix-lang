// mod analysis;
mod arena;
mod ast;
mod errors;
mod lexer;
// mod llvm;
mod parser;
mod strings;
// mod tir;

fn main() {
    test();
}

fn test() {
    let test_file = "test.rix";
    let src = std::fs::read_to_string(test_file).unwrap();

    let mut interner = strings::Interner::default();

    let mut errors = errors::Errors::new();
    let mut parser = parser::Parser::new(&src, &mut interner, &mut errors);
    parser.parse();

    let ast = parser.finish();
    let view = ast.view(&interner, &src);
    // let analysis = analysis::analyze(&ast, &interner, &mut errors);

    println!("Parsed AST:\n{}", view);

    // let analysis = analysis::analyse(view, &mut errors);
    // let tir = tir::lower(view, analysis);
    // println!("Lowered TIR:\n{}", tir.view(&interner));

    let error_printer = errors::ErrorPrinter::new(test_file, &src);
    errors.sort();
    for error in &errors {
        error_printer.print(error);
    }

    // llvm::lower(tir.view(&interner));
}
