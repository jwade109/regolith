#![allow(warnings)]

use argparse::{ArgumentParser, Store};
use crate::lexer::lex_markdown;
use crate::parser::{parse_to_ast, print_tree};
use crate::compiler::compile;

mod lexer;
mod compiler;
mod moonbase;
mod parser;

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

    if let Ok(t) = tree
    {
        print_tree(&t);
    }
    else
    {
        println!("{:?}", tree);
    }

    // println!("{} -> {}", &inpath, &outpath);
    // compile(&inpath, &outpath)?;

    // println!("Finished with no errors.");

    Ok(())
}
