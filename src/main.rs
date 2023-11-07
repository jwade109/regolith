#![allow(dead_code, unused)]

use std::env;
use anyhow::Result;

pub mod compiler;

use crate::compiler::*;

extern crate argparse;

use argparse::{ArgumentParser, StoreTrue, Store};

fn main() -> anyhow::Result<()>
{
    let mut verbose = false;

    let mut inpath = String::new();
    let mut outpath = String::new();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Regolith compiler.");
        ap.refer(&mut inpath)
            .add_argument("regfile", Store, "Input regolith file")
            .required();
        ap.refer(&mut outpath)
            .add_argument("songfile", Store, "Output sound file")
            .required();
        ap.parse_args_or_exit();
    }

    compile(&inpath, &outpath)?;

    println!("Finished with no errors.");

    Ok(())
}
