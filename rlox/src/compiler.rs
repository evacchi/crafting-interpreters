use scanner::Scanner;
use scanner::TokenType;

pub fn compile(source: String) {
    let mut line = 0;
    let mut scanner = Scanner::new(source);
    loop {
        let token = scanner.scan();
        if token.line != 0 && token.line != line {
            print!("{:4} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }
        print!("{:?} '{}'\n", token.tpe, token.text);

        if token.tpe == TokenType::TOKEN_EOF {
            break;  
        } 
    }
}