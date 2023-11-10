use anyhow::Result;
use std::collections::HashMap;
use fraction::{Fraction, ToPrimitive};
use anyhow::bail;

use crate::lexer::*;
use crate::moonbase::{to_moonbase_str, MoonbaseNote, generate_moonbase};

#[derive(Debug)]
struct Sequence
{
    id: u8,
    notes: Vec<MoonbaseNote> // TODO public?
}

#[derive(Debug)]
pub struct Section
{
    name: String,
    tracks: Vec<Track>
}

#[derive(Debug)]
pub struct Track
{
    name: String,
    measures: Vec<Vec<Token>>
}

impl Track
{
    fn beats(&self) -> Fraction
    {
        self.measures.iter().map(|m| count_beats(m)).sum()
    }
}

fn beats_to_millis(beats: &Fraction, bpm: u16) -> Option<i32>
{
    Some((beats.to_f64()? * 60000.0 / bpm as f64) as i32)
}

fn count_beats(tokens: &[Token]) -> Fraction
{
    tokens.iter().map(|t|
    {
        if let Token::Note(r) = t
        {
            r.beats
        }
        else
        {
            Fraction::new(0u64, 1u64)
        }
    }).sum()
}

#[test]
fn how_many_beats_are_there()
{
    assert_eq!(Fraction::new(0u64, 1u64), count_beats(&[]));
    assert_eq!(Fraction::new(3u64, 4u64), count_beats(&
    [
        Token::Note(RegoNote
        {
            prefix: String::new(),
            suffix: String::new(),
            beats: Fraction::new(2u64, 4u64),
        }),
        Token::Note(RegoNote
        {
            prefix: String::new(),
            suffix: String::new(),
            beats: Fraction::new(1u64, 4u64),
        })
    ]));
}

pub fn parse_tokens(tokens: &Vec<(Literal, Token)>) -> Result<Vec<Section>>
{
    let just_tokens : Vec<Token> = tokens.iter().map(|(l, t)| t.clone()).collect();

    let sections : Vec<Section> = split_by_section(&just_tokens).iter().map(|(sn, st)|
    {
        Section
        {
            name: sn.to_string(),
            tracks: split_by_track(&st).iter().map(|(n, t)|
            {
                Track
                {
                    name: n.to_string(),
                    measures: split_by_measure(&t).into_iter().collect()
                }
            }).collect()
        }
    }).collect();

    for section in &sections
    {
        println!("SECTION `{}`", section.name);
        for track in &section.tracks
        {
            println!(" TRACK `{}`", track.name);
            for measure in &track.measures
            {
                println!("     ~");
                for token in measure.iter()
                {
                    println!("    {:?}", token);
                }
            }
        }
    }

    Ok(vec![])
}

fn generate_from_sequences(sequences: &Vec<Sequence>) -> Result<()>
{
    for seq in sequences
    {
        let mb = seq.notes.iter().map(to_moonbase_str)
            .collect::<Vec<String>>().join("");
        let _path = generate_moonbase(&mb)?;
    }
    Ok(())
}

fn split_by_measure(tokens: &[Token]) -> Vec<Vec<Token>>
{
    let mut measures = vec![];

    let mut current_measure = vec![];

    for token in tokens
    {
        if let Token::MeasureBar() = token
        {
            if !current_measure.is_empty()
            {
                measures.push(current_measure.clone());
                current_measure.clear();
            }
        }
        else
        {
            current_measure.push(token.clone());
        }
    }
    if !current_measure.is_empty()
    {
        measures.push(current_measure);
    }

    return measures.into_iter().filter(|m| !m.is_empty()).collect();
}

fn split_by_track(tokens: &[Token]) -> Vec<(String, Vec<Token>)>
{
    let mut tracks : HashMap<String, Vec<Token>> = HashMap::new();

    let mut current_track = String::new();

    for token in tokens
    {
        if let Token::Track(name) = token
        {
            current_track = name.to_string();
        }
        else
        {
            tracks.entry(current_track.clone())
                .or_insert(vec![]).push(token.clone());
        }
    }

    tracks.into_iter().filter(|(n, t)| !t.is_empty()).collect()
}

fn split_by_section(tokens: &[Token]) -> Vec<(String, Vec<Token>)>
{
    let mut sections = vec![];

    let mut sname = String::new();
    let mut stok = vec![];

    for token in tokens
    {
        if let Token::Section(name) = token
        {
            if !stok.is_empty()
            {
                sections.push((sname, stok.clone()));
                stok.clear();
            }
            sname = name.to_string();
        }
        else
        {
            stok.push(token.clone());
        }
    }
    if !stok.is_empty()
    {
        sections.push((sname, stok))
    }

    sections.into_iter().filter(|(n, t)| !t.is_empty()).collect()
}

pub fn compile(inpath: &str, outpath: &str) -> Result<()>
{
    let tokens = lex_file(inpath)?;
    let sections = parse_tokens(&tokens)?;
    Ok(())
}

#[allow(unused_macros)]
macro_rules! assert_result
{
    ($to_test: expr, $on_ok: expr) =>
    {
        match $to_test
        {
            Ok(result) => assert_eq!(result, $on_ok),
            Err(error) =>
            {
                println!("Error: {:?}", error);
                assert!(false);
            }
        }
    }
}

#[test]
fn compile_songs()
{
    assert_result!(compile("examples/batman.reg",     "rust_songs/batman.wav"),     ());
    assert_result!(compile("examples/campfire.reg",   "rust_songs/campfire.wav"),   ());
    assert_result!(compile("examples/choir_test.reg", "rust_songs/choir_test.wav"), ());
    assert_result!(compile("examples/dynamics.reg",   "rust_songs/dynamics.wav"),   ());
    assert_result!(compile("examples/hbjm.reg",       "rust_songs/hbjm.wav"),       ());
    assert_result!(compile("examples/regularity.reg", "rust_songs/regularity.wav"), ());
    assert_result!(compile("examples/scales.reg",     "rust_songs/scales.wav"),     ());
    assert_result!(compile("examples/mariah.reg",     "rust_songs/mariah.wav"),     ());

    assert_result!(compile(
        "examples/thelionsleepstonight.reg",
        "/tmp/thelionsleepstonight.wav"), ());
}
