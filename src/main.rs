#![allow(warnings)]

use argparse::{ArgumentParser, Store};
use semantics::Composition;
use crate::semantics::do_semantics;
use crate::types::{CompileError, CompileResult, Literal, Token};
use crate::lexer::{lex_multiline_string, lex_markdown};
use crate::parser::{parse_to_ast, print_tree, print_error};
use crate::codegen::generate_mb_code;
use crate::moonbase::create_dir; // TODO move
use std::path::Path;
// use crate::compiler::compile;

mod types;
mod lexer;
mod compiler;
mod moonbase;
mod parser;
mod semantics;
mod codegen;

enum CompileInput<'a>
{
    StringLiteral(&'a String),
    Markdown(&'a Path)
}

fn print_composition(comp: &Composition)
{
    for section in &comp.sections
    {
        for measure in &section.measures
        {
            // TODO
        }
    }
}

fn compile(input: &CompileInput, build_root: &Path) -> CompileResult<()>
{
    create_dir(&build_root)?;

    let build_name = match input
    {
        CompileInput::StringLiteral(s) =>
        {
            let hash = md5::compute(&s);
            format!("string-literal-{:x}", hash)
        },
        CompileInput::Markdown(p) =>
        {
            let bytes = std::fs::read(p).unwrap();
            let hash = md5::compute(&bytes);

            let err = || {
                CompileError::Generic("Bad filename".to_string())
            };

            let file_name = p.file_name().ok_or_else(err)?.to_str().ok_or_else(err)?;

            format!("{}-{:x}", file_name, hash)
        }
    };

    let build_dir = build_root.join(build_name);

    println!("Build directory: {}", build_dir.display());

    create_dir(&build_dir)?;

    let tokens = match input
    {
        CompileInput::StringLiteral(s) =>
        {
            lex_multiline_string(s)
        },
        CompileInput::Markdown(p) =>
        {
            lex_markdown(p)
        }
    }?;

    let tree = parse_to_ast(&tokens)?;
    let comp = do_semantics(&tree)?;
    print_composition(&comp);
    generate_mb_code(&comp, &build_dir)?;
    Ok(())
}

fn main() -> Result<(), ()>
{
    let mut inpath = String::new();
    let mut source = String::new();
    let mut build_dir = String::new();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Regolith compiler.");
        ap.refer(&mut inpath)
            .add_option(&["--path"], Store, "Input regolith file");
        ap.refer(&mut source)
            .add_option(&["--source"], Store, "Regolith source to parse");
        ap.refer(&mut build_dir)
            .add_argument("build-dir", Store, "Output build directory")
            .required();
        ap.parse_args_or_exit();
    }

    let dir = Path::new(&build_dir);

    let res = if !inpath.is_empty()
    {
        compile(&CompileInput::Markdown(Path::new(&inpath)), dir)
    }
    else if !source.is_empty()
    {
        compile(&CompileInput::StringLiteral(&source), dir)
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
