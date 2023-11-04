#![allow(dead_code, unused)]

use std::env;
use std::fs::read_to_string;
use regex::Regex;
use regex_macro::regex;

extern crate argparse;

use argparse::{ArgumentParser, StoreTrue, Store};

struct Literal
{
    serialno: i32,
    literal: String,
    filename: String,
    lineno: i32,
    colno: i32,
}

#[derive(Debug)]
struct TrackDirective
{
    track_id: i32
}

struct Note
{
    prefix: String,
    suffix: String,
    beats: (i32, i32),
    literal: Literal
}

#[derive(Debug)]
enum Token
{
    Track(TrackDirective)
}

fn read_literals(filename: &str) -> Vec<String>
{
    let mut result = Vec::new();

    for line in read_to_string(filename).unwrap().lines()
    {
        if line.is_empty() || line.starts_with("#")
        {
            continue;
        }
        for c in line.to_string().split(" ")
        {
            result.push(c.to_string());
        }
    }

    return result;
}

fn tokenize_literal(literal: &str) -> Option<Token>
{
    let BPM_TOKEN_REGEX = regex!(r"(\d+)BPM$");
    let BEAT_ASSERT_TOKEN_REGEX = regex!(r"\@(\d+)$");
    let TRACK_TOKEN_REGEX = regex!(r"TRACK(\d+)$");
    let PITCH_TOKEN_REGEX = regex!(r"([A-G]\d?#?)$");
    let SCALE_DEGREE_REGEX = regex!(r"(\d+)([#b])?$");
    let PHONEME_TOKEN_REGEX = regex!(r"([a-z\-\.]+)(:(\d+))?(\/(\d+))?$");
    let SCALE_DECLARATION_REGEX = regex!(r"([A-G]\d*[#b]?)\[?((\d+)|PENTA|MAJOR|MINOR|CHROM)\]?$");
    let DYNAMICS_REGEX = regex!(r"FORTISSIMO|FORTE|MEZZOFORTE|MEZZOPIANO|PIANO|PIANISSIMO");



    return None;
}

fn main()
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

    println!("{}, {}", inpath, outpath);

    for lit in read_literals(&inpath)
    {
        let token = tokenize_literal(&lit);
        println!("{:?} -> {:?}", &lit, &token);
    }
}
