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

#[derive(Debug)]
struct TempoDirective
{
    bpm: i32
}

#[derive(Debug)]
struct PitchDirective
{
    tone_str: String
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
    Track(TrackDirective),
    Tempo(TempoDirective),
    Pitch(PitchDirective),
    Phoneme(String),
    Repeat(),
    BeatAssert(),
    Scale(),
    ScaleDegree(),
    Dynamic(),
    MeasureBar()
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
            if !c.is_empty()
            {
                result.push(c.to_string());
            }
        }
    }

    return result;
}

macro_rules! parse_rule
{
    ($lit: expr, $re: expr, $callable: expr) => {
        match $re.captures($lit)
        {
            Some(caps) => { return Some($callable(caps)); }
            _ => ()
        };
    }
}

fn tokenize_literal(literal: &str) -> Option<Token>
{
    let measure_bar_re = regex!(r"\|");
    let repeat_token_re = regex!(r"\:\|");
    let beat_assert_re = regex!(r"@(\d+)");
    let bpm_token_re = regex!(r"(\d+)BPM$");
    let beat_asser_token_re = regex!(r"\@(\d+)$");
    let track_token_re = regex!(r"TRACK(\d+)$");
    let pitch_token_re = regex!(r"([A-G]\d?#?)$");
    let scale_degree_re = regex!(r"(\d+)([#b])?$");
    let phoneme_token_re = regex!(r"([a-z\-\.]+)(:(\d+))?(\/(\d+))?$");
    let scale_decl_re = regex!(r"([A-G]\d*[#b]?)\[?((\d+)|PENTA|MAJOR|MINOR|CHROM)\]?$");
    let dynamic_decl_re = regex!(r"FORTISSIMO|FORTE|MEZZOFORTE|MEZZOPIANO|PIANO|PIANISSIMO");

    parse_rule!(&literal, bpm_token_re, |cap: regex::Captures|
    {
        let bpm : i32 = cap.get(1).unwrap().as_str().parse().unwrap();
        return Token::Tempo(TempoDirective{bpm: bpm});
    });

    parse_rule!(&literal, track_token_re, |cap: regex::Captures|
    {
        let idx : i32 = cap.get(1).unwrap().as_str().parse().unwrap();
        return Token::Track(TrackDirective{track_id: idx});
    });

    parse_rule!(&literal, pitch_token_re, |cap: regex::Captures|
    {
        let s : String = cap.get(1).unwrap().as_str().to_string();
        return Token::Pitch(PitchDirective{tone_str: s});
    });

    parse_rule!(&literal, phoneme_token_re, |cap: regex::Captures|
    {
        let s : String = cap.get(0).unwrap().as_str().to_string();
        return Token::Phoneme(s);
    });

    parse_rule!(&literal, repeat_token_re, |cap: regex::Captures|
    {
        return Token::Repeat();
    });

    parse_rule!(&literal, beat_assert_re, |cap: regex::Captures|
    {
        return Token::BeatAssert();
    });

    parse_rule!(&literal, scale_decl_re, |cap: regex::Captures|
    {
        return Token::Scale();
    });

    parse_rule!(&literal, dynamic_decl_re, |cap: regex::Captures|
    {
        return Token::Dynamic();
    });

    parse_rule!(&literal, scale_degree_re, |cap: regex::Captures|
    {
        return Token::ScaleDegree();
    });

    parse_rule!(&literal, measure_bar_re, |cap: regex::Captures|
    {
        return Token::MeasureBar();
    });

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
        match token
        {
            Some(ref t) => println!("{:?} -> {:?}", &lit, &t),
            None =>
            {
                println!("Bad token: {:?}", &lit);
                std::process::exit(1);
            }
        }
    }

    println!("Finished with no errors.");
}
