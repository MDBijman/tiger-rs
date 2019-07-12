/*
 * Copyright (c) 2017-2019 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

mod ast;
mod env;
mod error;
mod escape;
mod frame;
mod gen;
mod ir;
mod lexer;
mod parser;
mod position;
mod semant;
mod symbol;
mod temp;
mod terminal;
mod token;
mod types;

use std::env::args;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;

use env::Env;
use error::Error;
use escape::find_escapes;
use frame::Fragment;
use frame::x86_64::X86_64;
use lexer::Lexer;
use symbol::{Strings, Symbols};
use parser::Parser;
use semant::SemanticAnalyzer;
use terminal::Terminal;

fn main() {
    let strings = Rc::new(Strings::new());
    let mut symbols = Symbols::new(Rc::clone(&strings));
    if let Err(error) = drive(strings, &mut symbols) {
        let terminal = Terminal::new();
        if let Err(error) = error.show(&symbols, &terminal) {
            eprintln!("Error printing errors: {}", error);
        }
    }
}

fn drive(strings: Rc<Strings>, symbols: &mut Symbols<()>) -> Result<(), Error> {
    let mut args = args();
    args.next();
    if let Some(filename) = args.next() {
        let file = BufReader::new(File::open(&filename)?);
        let file_symbol = symbols.symbol(&filename);
        let lexer = Lexer::new(file, file_symbol);
        let main_symbol = symbols.symbol("main");
        let mut parser = Parser::new(lexer, symbols);
        let ast = parser.parse()?;
        let escape_env = find_escapes(&ast, Rc::clone(&strings));
        let mut env = Env::<X86_64>::new(&strings, escape_env);
        let semantic_analyzer = SemanticAnalyzer::new(&mut env, Rc::clone(&strings));
        let fragments = semantic_analyzer.analyze(main_symbol, ast)?;

        for fragment in &fragments {
            match fragment {
                Fragment::Function { body, .. } => println!("{:#?}", body),
                Fragment::Str(_, string) => println!("String {}", string),
            }
        }
    }
    Ok(())
}
