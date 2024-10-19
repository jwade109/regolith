#![allow(warnings)]

use argparse::{ArgumentParser, Store};
use lexer::lex_multiline_string;
use semantics::do_semantics;
use types::{CompileResult, CompileError, Literal, Token};
use crate::lexer::lex_markdown;
use crate::parser::{parse_to_ast, print_tree, print_error};
// use crate::compiler::compile;

mod types;
mod lexer;
mod compiler;
mod moonbase;
mod parser;
mod semantics;

fn compile(tokens: &Vec<(Literal, Token)>) -> CompileResult<()>
{
    let tree = parse_to_ast(&tokens)?;

    print_tree(&tree);

    let res = do_semantics(&tree)?;

    for section in res.sections
    {
        println!("{}", section.to_string());
    }

    Ok(())
}

fn compile_markdown(inpath: &str) -> CompileResult<()>
{
    let tokens = lex_markdown(&inpath)?;
    compile(&tokens)
}

fn compile_string(source: &str) -> CompileResult<()>
{
    let tokens = lex_multiline_string(source)?;
    compile(&tokens)
}

fn main() -> Result<(), ()>
{
    let mut inpath = String::new();
    let mut outpath = String::new();
    let mut source = String::new();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Regolith compiler.");
        ap.refer(&mut inpath)
            .add_option(&["--path"], Store, "Input regolith file");
        ap.refer(&mut source)
            .add_option(&["--source"], Store, "Regolith source to parse");
        ap.parse_args_or_exit();
    }

    let res = if !inpath.is_empty()
    {
        compile_markdown(&inpath)
    }
    else if !source.is_empty()
    {
        compile_string(&source)
    }
    else
    {
        println!("Requires --path or --source");
        return Err(());
    };

    match res
    {
        Ok(_) => Ok(()),
        Err(e) => { print_error(&e); Err(()) },
    }
}
