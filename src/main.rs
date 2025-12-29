pub mod ast;
pub mod lexer;

fn main() {
    let test_file = "test.rix";
    let test_data = std::fs::read_to_string(test_file).unwrap();

    let mut lexer = lexer::Lexer::new(&test_data);
    loop {
        let token = lexer.advance();
        println!("{:?}", token);

        if token.kind.is_eof() {
            break;
        }
    }
}
