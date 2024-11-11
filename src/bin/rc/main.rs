#![allow(warnings)]

use regolith::compiler::{compile, CompileInput};
use regolith::parser::print_error;
use argparse::{ArgumentParser, Store};
use std::path::Path;

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
