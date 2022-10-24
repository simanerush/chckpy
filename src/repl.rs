use std::io::Write;

use crate::ast::Prgm;
use crate::eval::{Ast, Context};

/// Run the REPL.
///
/// # Errors
/// If the code is wrong.
pub fn repl() -> anyhow::Result<()> {
    let mut source = String::new();
    let mut ctx = Context::default();
    loop {
        // TODO: handle EOF
        print!(">>> ");
        std::io::stdout().flush().expect("can flush stdout");
        if std::io::stdin().read_line(&mut source).is_ok() {
            let expn: Expn = source.parse()?;
            if let Ok(n) = prgm {
                println!("{}", n);
            } else {
                let tokens = Tokenizer::lex(source.as_str())?;
                Stmt::parse_and_eval(tokens, &mut ctx)?;
            }
        }
        source.clear();
    }
}
