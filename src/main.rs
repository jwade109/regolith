#![allow(warnings)]

use argparse::{ArgumentParser, Store};
use semantics::do_semantics;
use crate::lexer::lex_markdown;
use crate::parser::{parse_to_ast, print_tree, print_error};
// use crate::compiler::compile;

mod types;
mod lexer;
mod compiler;
mod moonbase;
mod parser;
mod semantics;

fn main() -> Result<(), ()>
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

    let tokens = match lex_markdown(&inpath)
    {
        Ok(tokens) => tokens,
        Err(error) =>
        {
            print_error(&error);
            return Err(());
        },
    };

    let tree = match parse_to_ast(&tokens)
    {
        Ok(tree) => tree,
        Err(error) =>
        {
            print_error(&error);
            return Err(());
        }
    };

    print_tree(&tree);

    let res = match do_semantics(&tree)
    {
        Ok(comp) => comp,
        Err(error) =>
        {
            print_error(&error);
            return Err(());
        }
    };

    for section in res.sections
    {
        println!("{}", section.to_string());
    }

    return Ok(());

    // do_semantics(&tree);

    // println!("{} -> {}", &inpath, &outpath);
    // compile(&inpath, &outpath)?;

    // println!("Finished with no errors.");

    // Ok(())
}
