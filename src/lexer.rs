use std::fs::read_to_string;
use fraction::{Fraction, ToPrimitive};
use regex_macro::regex;
use std::collections::HashMap;
use anyhow::{Result, Context, bail};

use crate::moonbase::MoonbaseNote;
use crate::compiler::Sequence;

static PITCH_MAP : [(&str, u8); 49] =
[
    // ambiguous map -- get rid of it!
    ("A"  , 10),
    ("A#" , 11),
    ("B"  , 12),
    ("C"  , 13),
    ("C#" , 14),
    ("D"  , 15),
    ("D#" , 16),
    ("E"  , 17),
    ("F"  , 18),
    ("F#" , 19),
    ("G"  , 20),
    ("G#" , 21),

    ("C1" , 1),
    ("C1#", 2),
    ("D1" , 3),
    ("D1#", 4),
    ("E1" , 5),
    ("F1" , 6),
    ("F1#", 7),
    ("G1" , 8),
    ("G1#", 9),
    ("A1" , 10),
    ("A1#", 11),
    ("B1" , 12),
    ("C2" , 13),
    ("C2#", 14),
    ("D2" , 15),
    ("D2#", 16),
    ("E2" , 17),
    ("F2" , 18),
    ("F2#", 19),
    ("G2" , 20),
    ("G2#", 21),
    ("A2" , 22),
    ("A2#", 23),
    ("B2" , 24),
    ("C3" , 25),
    ("C3#", 26),
    ("D3" , 27),
    ("D3#", 28),
    ("E3" , 29),
    ("F3" , 30),
    ("F3#", 31),
    ("G3" , 32),
    ("G3#", 33),
    ("A3" , 34),
    ("A3#", 35),
    ("B3" , 36),
    ("C4" , 37)
];

fn pitch_string_to_id(pitch: &str) -> Result<u8>
{
    let (_, i) = PITCH_MAP.iter().find(|(s, _)| *s == pitch)
        .context(format!("Bad pitch string: `{}`", pitch))?;
    Ok(*i)
}

#[test]
fn pitch_string_conversions()
{
    assert_eq!(pitch_string_to_id("C1").ok(),  Some(1));
    assert_eq!(pitch_string_to_id("D2#").ok(), Some(16));
    assert_eq!(pitch_string_to_id("A2#").ok(), Some(23));
    assert_eq!(pitch_string_to_id("G3").ok(),  Some(32));
    assert_eq!(pitch_string_to_id("C4").ok(),  Some(37));
    assert_eq!(pitch_string_to_id("").ok(),    None);
    assert_eq!(pitch_string_to_id("J3").ok(),  None);
    assert_eq!(pitch_string_to_id("Bb").ok(),  None);
}

static NAMED_SCALE_MAP : [(&str, &[u8; 12]); 4] =
[
    ("MAJOR", &[2, 2, 1, 2, 2, 2, 1, 0, 0, 0, 0, 0]),
    ("MINOR", &[2, 1, 2, 2, 1, 2, 2, 0, 0, 0, 0, 0]),
    ("PENTA", &[2, 2, 3, 2, 3, 0, 0, 0, 0, 0, 0, 0]),
    ("CHROM", &[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1])
];

fn get_named_scale_steps(scale: &str) -> Option<Vec<u8>>
{
    let (_, s) = NAMED_SCALE_MAP.iter().find(|(n, _)| *n == scale)?;
    let v : Vec<u8> = s.iter().cloned().filter(|x| *x > 0u8).collect::<Vec<_>>();
    Some(v)
}

#[test]
fn named_scale_lookup()
{
    assert_eq!(get_named_scale_steps("MAJOR"), Some(vec![2, 2, 1, 2, 2, 2, 1]));
    assert_eq!(get_named_scale_steps("MINOR"), Some(vec![2, 1, 2, 2, 1, 2, 2]));
    assert_eq!(get_named_scale_steps("PENTA"), Some(vec![2, 2, 3, 2, 3]));
    assert_eq!(get_named_scale_steps("CHROM"), Some(vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
    assert_eq!(get_named_scale_steps("DINGO"), None);
    assert_eq!(get_named_scale_steps(""),      None);
}

#[derive(Debug, PartialEq, Eq)]
pub struct Literal
{
    literal: String,
    filename: String,
    lineno: usize,
    colno: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RegoNote
{
    prefix: String,
    suffix: String,
    beats: Fraction
}

#[derive(Debug, PartialEq, Eq)]
pub enum DynamicLevel
{
    Pianissimo,
    Piano,
    Mezzopiano,
    Mezzoforte,
    Forte,
    Fortissimo
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Scale
{
    tone_id: u8,
    steps: Vec<u8>
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token
{
    Track(u8),
    Tempo(u16),
    AbsolutePitch(u8),
    Note(RegoNote),
    Repeat(bool),
    BeatAssert(i32),
    Scale(Scale),
    ScaleDegree(i32),
    Dynamic(DynamicLevel),
    MeasureBar()
}

fn read_literals_from_file(filename: &str) -> Result<Vec<Literal>>
{
    let mut result = Vec::new();

    for (lineno, line) in read_to_string(filename)?.lines().enumerate()
    {
        if line.is_empty() || line.starts_with('#')
        {
            continue;
        }
        for c in line.to_string().split(' ')
        {
            if !c.is_empty()
            {
                let l = Literal
                {
                    colno: 0,
                    filename: filename.to_string(),
                    lineno,
                    literal: c.to_string()
                };
                result.push(l);
            }
        }
    }

    Ok(result)
}

macro_rules! lex_rule
{
    ($lit: expr, $re: expr, $callable: expr) => {
        if let Some(captures) = $re.captures($lit)
        {
            let v : Vec<Option<String>> = captures.iter().map(|cap|
            {
                match cap
                {
                    Some(c) => Some(c.as_str().to_string()),
                    None    => None
                }
            })
            .collect();

            return $callable(&v);
        }
    }
}

#[allow(unused_macros)]
macro_rules! lex_assert
{
    ($string: expr, $expect: expr) =>
    {
        match lex_literal($string)
        {
            Ok(result) => assert_eq!(result, $expect),
            Err(error) =>
            {
                println!("Error: {:?}", error);
                assert!(false);
            }
        }
    }
}

fn parse_dynamic_level(level: &str) -> Option<DynamicLevel>
{
    match level
    {
        "PIANISSIMO" => Some(DynamicLevel::Pianissimo),
        "PIANO"      => Some(DynamicLevel::Piano),
        "MEZZOPIANO" => Some(DynamicLevel::Mezzopiano),
        "MEZZOFORTE" => Some(DynamicLevel::Mezzoforte),
        "FORTE"      => Some(DynamicLevel::Forte),
        "FORTISSIMO" => Some(DynamicLevel::Fortissimo),
        _            => None
    }
}

#[test]
fn dynamic_lexing()
{
    lex_assert!("PIANISSIMO", Token::Dynamic(DynamicLevel::Pianissimo));
    lex_assert!("PIANO",      Token::Dynamic(DynamicLevel::Piano));
    lex_assert!("MEZZOPIANO", Token::Dynamic(DynamicLevel::Mezzopiano));
    lex_assert!("MEZZOFORTE", Token::Dynamic(DynamicLevel::Mezzoforte));
    lex_assert!("FORTE",      Token::Dynamic(DynamicLevel::Forte));
    lex_assert!("FORTISSIMO", Token::Dynamic(DynamicLevel::Fortissimo));
}

fn get_nth_capture(captures: &[Option<String>], i: usize) -> Result<String>
{
    Ok(captures.get(i).context("No nth element")?.clone()
                      .context("Nth element is None")?.clone())
}

fn lex_literal(literal: &str) -> Result<Token>
{
    let measure_bar_re = regex!(r"^\|");
    let start_repeat_re = regex!(r"^\[:");
    let stop_repeat_re = regex!(r"^:\]");
    let beat_assert_re = regex!(r"^@(\d+)$");
    let bpm_token_re = regex!(r"^(\d+)BPM$");
    let track_token_re = regex!(r"^\[(\d+)\]$");
    let pitch_token_re = regex!(r"^[A-Z]\d?#?$");
    let scale_degree_re = regex!(r"^(\d+)([#b])?$");
    let note_token_re = regex!(r"^([a-z\.]+)\-?([a-z\.]+)?(:(\d+))?(\/(\d+))?$");
    let scale_decl_re = regex!(r"^([A-G]\d*[#b]?)(\[(\d+)\]|PENTA|MAJOR|MINOR|CHROM)?$");
    let dynamic_decl_re = regex!(r"^FORTISSIMO|FORTE|MEZZOFORTE|MEZZOPIANO|PIANO|PIANISSIMO");
    let rest_decl_re = regex!(r"^-(:(\d+))?(\/(\d+))?$");

    lex_rule!(&literal, bpm_token_re, |cap: &[Option<String>]|
    {
        let bpm : u16 = get_nth_capture(cap, 1)?.parse().context("Bad regex")?;
        Ok(Token::Tempo(bpm))
    });

    lex_rule!(&literal, track_token_re, |cap: &[Option<String>]|
    {
        let idx : u8 = get_nth_capture(cap, 1)?.parse().context("Bad regex")?;
        Ok(Token::Track(idx))
    });

    lex_rule!(&literal, pitch_token_re, |cap: &[Option<String>]|
    {
        let s : String = get_nth_capture(cap, 0)?;
        let id : u8 = pitch_string_to_id(&s)?;
        Ok(Token::AbsolutePitch(id))
    });

    lex_rule!(&literal, note_token_re, |cap: &[Option<String>]|
    {
        let numer : u64 = match cap[4].as_ref()
        {
            Some(s) => s.parse().unwrap(),
            None    => 1
        };

        let denom : u64 = match cap[6].as_ref()
        {
            Some(s) => s.parse().unwrap(),
            None    => 1
        };

        let n = RegoNote
        {
            prefix: cap[1].as_ref().unwrap_or(&"".to_string()).clone(),
            suffix: cap[2].as_ref().unwrap_or(&"".to_string()).clone(),
            beats: Fraction::new(numer, denom)
        };
        Ok(Token::Note(n))
    });

    lex_rule!(&literal, start_repeat_re, |_: &[Option<String>]|
    {
        Ok(Token::Repeat(true))
    });

    lex_rule!(&literal, stop_repeat_re, |_: &[Option<String>]|
    {
        Ok(Token::Repeat(false))
    });

    lex_rule!(&literal, beat_assert_re, |cap: &[Option<String>]|
    {
        let beats : i32 = get_nth_capture(cap, 1)?.parse().unwrap();
        Ok(Token::BeatAssert(beats))
    });

    lex_rule!(&literal, scale_decl_re, |cap: &[Option<String>]|
    {
        let pitch_str = get_nth_capture(cap, 1)?;
        let tone_id = pitch_string_to_id(&pitch_str)?;
        let steps : Vec<u8> = if let Some(numbers) = cap.get(3).context("Bad regex")?
        {
            numbers.chars().map(|c| c.to_digit(10).unwrap() as u8).collect::<Vec<_>>()
        }
        else
        {
            get_named_scale_steps(&get_nth_capture(cap, 2)?).context("Bad regex")?
        };

        let s = Scale
        {
            tone_id,
            steps
        };

        Ok(Token::Scale(s))
    });

    lex_rule!(&literal, dynamic_decl_re, |cap: &[Option<String>]|
    {
        let level = parse_dynamic_level(cap[0].as_ref().unwrap());
        Ok(Token::Dynamic(level.unwrap()))
    });

    lex_rule!(&literal, scale_degree_re, |cap: &[Option<String>]|
    {
        let d : i32 = get_nth_capture(cap, 1)?.parse().context("Bad regex")?;
        Ok(Token::ScaleDegree(d))
    });

    lex_rule!(&literal, measure_bar_re, |_: &[Option<String>]|
    {
        Ok(Token::MeasureBar())
    });

    lex_rule!(&literal, rest_decl_re, |cap: &[Option<String>]|
    {
        let numer : u64 = match cap[2].as_ref()
        {
            Some(s) => s.parse().unwrap(),
            None    => 1
        };

        let denom : u64 = match cap[4].as_ref()
        {
            Some(s) => s.parse().unwrap(),
            None    => 1
        };

        let n = RegoNote
        {
            prefix: "_".to_string(),
            suffix: "".to_string(),
            beats: Fraction::new(numer, denom)
        };

        Ok(Token::Note(n))
    });

    bail!("No rule to lex symbol `{}`", &literal);
}

pub fn lex_file(inpath: &str) -> Result<Vec<Token>>
{
    let literals = read_literals_from_file(inpath)?;
    let mut ret = vec![];
    for lit in literals
    {
        let token = lex_literal(&lit.literal)?;
        ret.push(token);
    }
    Ok(ret)
}

fn beats_to_millis(beats: &Fraction, bpm: u16) -> Option<i32>
{
    Some((beats.to_f64()? * 60000.0 / bpm as f64) as i32)
}

pub fn parse_tokens(tokens: &Vec<Token>) -> Result<Vec<Sequence>>
{
    let mut current_bpm : u16 = 120;
    let mut current_track : u8 = 0;
    let mut current_pitch = pitch_string_to_id("C2")?;
    let mut _current_scale = Scale
    {
        tone_id: pitch_string_to_id("C2")?,
        steps: get_named_scale_steps("MAJOR").context("Bad default scale")?
    };

    let mut tracks : HashMap<u8, Sequence> = HashMap::new();

    for t in tokens
    {
        match t
        {
            Token::Track(t) => current_track = *t,
            Token::Tempo(bpm) => current_bpm = *bpm,
            Token::AbsolutePitch(p) => current_pitch = *p,
            Token::Note(n) =>
            {
                let mb = MoonbaseNote
                {
                    prefix: n.prefix.clone(),
                    suffix: n.suffix.clone(),
                    dur_ms: beats_to_millis(&n.beats, current_bpm).context("Bad fraction")?,
                    tone_id: current_pitch
                };

                let seq = tracks.entry(current_track)
                    .or_insert(Sequence { id: current_track, notes: vec![] });
                seq.notes.push(mb);
            }
            Token::Repeat(_b) => (),
            Token::BeatAssert(_b) => (),
            Token::Scale(s) => _current_scale = s.clone(),
            Token::ScaleDegree(_d) => (),
            Token::Dynamic(_l) => (),
            Token::MeasureBar() => ()
        }
    }

    Ok(tracks.into_values().collect())
}

#[test]
fn note_lexing()
{
    lex_assert!("ih-s:3/2",
    Token::Note(RegoNote
    {
        prefix: "ih".to_string(),
        suffix: "s".to_string(),
        beats: Fraction::new(3u64, 2u64)
    }));

    lex_assert!("uh-n/2",
    Token::Note(RegoNote
    {
        prefix: "uh".to_string(),
        suffix: "n".to_string(),
        beats: Fraction::new(1u64, 2u64)
    }));

    lex_assert!("ne/3",
    Token::Note(RegoNote
    {
        prefix: "ne".to_string(),
        suffix: "".to_string(),
        beats: Fraction::new(1u64, 3u64)
    }));

    lex_assert!("-:12",
    Token::Note(RegoNote
    {
        prefix: "_".to_string(),
        suffix: "".to_string(),
        beats: Fraction::new(12u64, 1u64)
    }));
}

#[test]
fn absolute_pitch_lexing()
{
    lex_assert!("C", Token::AbsolutePitch(13));
    lex_assert!("D", Token::AbsolutePitch(15));
    lex_assert!("E", Token::AbsolutePitch(17));
}

#[test]
fn relative_pitch_lexing()
{
    lex_assert!("1", Token::ScaleDegree(1));
    lex_assert!("2", Token::ScaleDegree(2));
    lex_assert!("5", Token::ScaleDegree(5));
    lex_assert!("13", Token::ScaleDegree(13));

    assert!(lex_literal("-4").is_err());
    assert!(lex_literal("352d").is_err());
}

#[test]
fn scale_lexing()
{
    lex_assert!("CMAJOR", Token::Scale(Scale
    {
        tone_id: 13,
        steps: vec![2, 2, 1, 2, 2, 2, 1]
    }));

    lex_assert!("AMINOR", Token::Scale(Scale
    {
        tone_id: 10,
        steps: vec![2, 1, 2, 2, 1, 2, 2]
    }));

    lex_assert!("G#PENTA", Token::Scale(Scale
    {
        tone_id: 21,
        steps: vec![2, 2, 3, 2, 3]
    }));

    lex_assert!("D3#CHROM", Token::Scale(Scale
    {
        tone_id: 28,
        steps: vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
    }));

    // no brackets
    assert!(lex_literal("Fb3").is_err());

    // bad pitch
    assert!(lex_literal("K4[22211]").is_err());
}

#[test]
fn bar_lexing()
{
    lex_assert!("|", Token::MeasureBar());
}

#[test]
fn repeat_lexing()
{
    lex_assert!("[:", Token::Repeat(true));
    lex_assert!(":]", Token::Repeat(false));
}

#[test]
fn beats_assert_lexing()
{
    lex_assert!("@16",   Token::BeatAssert(16));
    lex_assert!("@32",   Token::BeatAssert(32));
    lex_assert!("@27",   Token::BeatAssert(27));
    lex_assert!("@0",    Token::BeatAssert(0));
    lex_assert!("@2452", Token::BeatAssert(2452));
    assert!(lex_literal("@-3").is_err());
}

#[test]
fn bpm_lexing()
{
    lex_assert!("120BPM",  Token::Tempo(120));
    lex_assert!("92BPM",   Token::Tempo(92));
    lex_assert!("1103BPM", Token::Tempo(1103));
    lex_assert!("0BPM",    Token::Tempo(0));

    assert!(lex_literal("-12BPM").is_err());
    assert!(lex_literal("CHEESEBPM").is_err());
    assert!(lex_literal("--BPM").is_err());
}

#[test]
fn track_lexing()
{
    lex_assert!("[0]",  Token::Track(0));
    lex_assert!("[1]",  Token::Track(1));
    lex_assert!("[2]",  Token::Track(2));
    lex_assert!("[9]",  Token::Track(9));
    lex_assert!("[12]", Token::Track(12));
}

#[test]
fn garbage_lexing()
{
    assert!(lex_literal("wefwe$234").is_err());
    assert!(lex_literal("dddFd").is_err());
    assert!(lex_literal("...--").is_err());
}
