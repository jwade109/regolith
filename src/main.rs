#![allow(warnings)]

use argparse::{ArgumentParser, Store};
use semantics::do_semantics;
use crate::lexer::lex_markdown;
use crate::parser::{parse_to_ast, print_tree, print_parse_error};
use crate::compiler::compile;

mod lexer;
mod compiler;
mod moonbase;
mod parser;
mod semantics;

fn main() -> anyhow::Result<()>
{
    let mut inpath = String::new();
    let mut outpath = String::new();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Regolith compiler.");
        ap.refer(&mut inpath)
            .add_argument("regfile", Store, "Input regolith file")
            .required();
        // ap.refer(&mut outpath)
        //     .add_argument("songfile", Store, "Output sound file")
        //     .required();
        ap.parse_args_or_exit();
    }

    let tokens = lex_markdown(&inpath).unwrap();
    let tree = parse_to_ast(&tokens);

    match tree
    {
        Ok(t) =>
        {
            print_tree(&t);
            println!("{:?}", do_semantics(&t));
        }
        Err(e) => print_parse_error(&e),
    }

    // println!("{} -> {}", &inpath, &outpath);
    // compile(&inpath, &outpath)?;

    // println!("Finished with no errors.");

    Ok(())
}
