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

// #[test]
// fn compile_songs()
// {
//     assert_result!(compile("examples/batman.reg",     "rust_songs/batman.wav"),     ());
//     assert_result!(compile("examples/campfire.reg",   "rust_songs/campfire.wav"),   ());
//     assert_result!(compile("examples/choir_test.reg", "rust_songs/choir_test.wav"), ());
//     assert_result!(compile("examples/dynamics.reg",   "rust_songs/dynamics.wav"),   ());
//     assert_result!(compile("examples/hbjm.reg",       "rust_songs/hbjm.wav"),       ());
//     assert_result!(compile("examples/regularity.reg", "rust_songs/regularity.wav"), ());
//     assert_result!(compile("examples/scales.reg",     "rust_songs/scales.wav"),     ());
//     assert_result!(compile("examples/mariah.reg",     "rust_songs/mariah.wav"),     ());

//     assert_result!(compile(
//         "examples/thelionsleepstonight.reg",
//         "/tmp/thelionsleepstonight.wav"), ());
// }
