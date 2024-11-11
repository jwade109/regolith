#![allow(warnings)]

use glob::glob;

use regolith::parser::print_error;
use regolith::compiler::{compile, CompileInput};
use std::path::Path;

fn main()
{
    let build_dir = Path::new("build/");

    for entry in glob("examples/*.md").unwrap()
    {
        if let Ok(e) = entry
        {
            let input = CompileInput::Markdown(&e);
            let res = compile(&input, build_dir);
            if let Err(r) = res
            {
                print_error(&r);
            }
        }
    }
}
