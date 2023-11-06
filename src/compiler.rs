#![allow(dead_code, unused)]

use fraction::{Fraction, ToPrimitive};
use std::fs::read_to_string;
use regex::Regex;
use regex_macro::regex;
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use anyhow::{Result, Context, bail};

extern crate reqwest;

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

pub fn pitch_string_to_id(pitch: &str) -> Result<u8>
{
    let (s, i) = PITCH_MAP.iter().find(|(s, i)| *s == pitch)
        .context(format!("Bad pitch string: `{}`", pitch))?;
    return Ok(*i);
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

pub fn get_named_scale_steps(scale: &str) -> Option<Vec<u8>>
{
    let (n, s) = NAMED_SCALE_MAP.iter().find(|(n, s)| *n == scale)?;
    let v : Vec<u8> = s.iter().cloned().filter(|x| *x > 0u8).collect::<Vec<_>>();
    return Some(v);
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
pub struct MoonbaseNote
{
    prefix: String,
    suffix: String,
    dur_ms: i32,
    tone_id: u8
}

#[derive(Debug, PartialEq, Eq)]
pub enum DynamicLevel
{
    PIANISSIMO,
    PIANO,
    MEZZOPIANO,
    MEZZOFORTE,
    FORTE,
    FORTISSIMO
}

#[derive(Debug, PartialEq, Eq)]
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
    Repeat(),
    BeatAssert(i32),
    Scale(Scale),
    ScaleDegree(i32),
    Dynamic(DynamicLevel),
    MeasureBar()
}

pub fn read_literals_from_file(filename: &str) -> Result<Vec<Literal>>
{
    let mut result = Vec::new();

    for (lineno, line) in read_to_string(filename)?.lines().enumerate()
    {
        if line.is_empty() || line.starts_with("#")
        {
            continue;
        }
        for c in line.to_string().split(" ")
        {
            if !c.is_empty()
            {
                let l = Literal
                {
                    colno: 0,
                    filename: filename.to_string(),
                    lineno: lineno,
                    literal: c.to_string()
                };
                result.push(l);
            }
        }
    }

    return Ok(result);
}

macro_rules! lex_rule
{
    ($lit: expr, $re: expr, $callable: expr) => {
        if let Some(caps) = $re.captures($lit)
        {
            let mut v : Vec<Option<String>> = vec![];

            for cap in caps.iter()
            {
                match cap
                {
                    Some(c) => v.push(Some(c.as_str().to_string())),
                    None => v.push(None)
                }
            }

            return $callable(v);
        }
    }
}

fn parse_dynamic_level(level: &str) -> Option<DynamicLevel>
{
    return match level
    {
        "PIANISSIMO" => Some(DynamicLevel::PIANISSIMO),
        "PIANO"      => Some(DynamicLevel::PIANO),
        "MEZZOPIANO" => Some(DynamicLevel::MEZZOPIANO),
        "MEZZOFORTE" => Some(DynamicLevel::MEZZOFORTE),
        "FORTE"      => Some(DynamicLevel::FORTE),
        "FORTISSIMO" => Some(DynamicLevel::FORTISSIMO),
        _            => None
    };
}

pub fn get_nth_capture(captures: &Vec<Option<String>>, i: usize) -> Result<String>
{
    return Ok(captures.get(i).context("No nth element")?.clone()
                             .context("Nth element is None")?.clone());
}

pub fn lex_literal(literal: &str) -> Result<Token>
{
    let measure_bar_re = regex!(r"^\|");
    let repeat_token_re = regex!(r"^\:\|");
    let beat_assert_re = regex!(r"^@(\d+)$");
    let bpm_token_re = regex!(r"^(\d+)BPM$");
    let track_token_re = regex!(r"^TRACK(\d+)$");
    let pitch_token_re = regex!(r"^[A-Z]\d?#?$");
    let scale_degree_re = regex!(r"^(\d+)([#b])?$");
    let note_token_re = regex!(r"^([a-z\.]+)\-?([a-z\.]+)?(:(\d+))?(\/(\d+))?$");
    let scale_decl_re = regex!(r"^([A-G]\d*[#b]?)(\[(\d+)\]|PENTA|MAJOR|MINOR|CHROM)?$");
    let dynamic_decl_re = regex!(r"^FORTISSIMO|FORTE|MEZZOFORTE|MEZZOPIANO|PIANO|PIANISSIMO");
    let rest_decl_re = regex!(r"^-(:(\d+))?(\/(\d+))?$");

    lex_rule!(&literal, bpm_token_re, |cap: Vec<Option<String>>|
    {
        let bpm : u16 = get_nth_capture(&cap, 1)?.parse().context("Bad regex")?;
        return Ok(Token::Tempo(bpm));
    });

    lex_rule!(&literal, track_token_re, |cap: Vec<Option<String>>|
    {
        let idx : u8 = get_nth_capture(&cap, 1)?.parse().context("Bad regex")?;
        return Ok(Token::Track(idx));
    });

    lex_rule!(&literal, pitch_token_re, |cap: Vec<Option<String>>|
    {
        let s : String = get_nth_capture(&cap, 0)?;
        let id : u8 = pitch_string_to_id(&s)?;
        return Ok(Token::AbsolutePitch(id));
    });

    lex_rule!(&literal, note_token_re, |cap: Vec<Option<String>>|
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
        return Ok(Token::Note(n));
    });

    lex_rule!(&literal, repeat_token_re, |cap: Vec<Option<String>>|
    {
        return Ok(Token::Repeat());
    });

    lex_rule!(&literal, beat_assert_re, |cap: Vec<Option<String>>|
    {
        let beats : i32 = get_nth_capture(&cap, 1)?.parse().unwrap();
        return Ok(Token::BeatAssert(beats));
    });

    lex_rule!(&literal, scale_decl_re, |cap: Vec<Option<String>>|
    {
        let pitch_str = get_nth_capture(&cap, 1)?;
        let tone_id = pitch_string_to_id(&pitch_str)?;
        let steps : Vec<u8> = if let Some(numbers) = cap.get(3).context("Bad regex")?
        {
            numbers.chars().map(|c| c.to_digit(10).unwrap() as u8).collect::<Vec<_>>()
        }
        else
        {
            get_named_scale_steps(&get_nth_capture(&cap, 2)?).context("Bad regex")?
        };

        let s = Scale
        {
            tone_id: tone_id,
            steps: steps
        };

        return Ok(Token::Scale(s));
    });

    lex_rule!(&literal, dynamic_decl_re, |cap: Vec<Option<String>>|
    {
        let level = parse_dynamic_level(cap[0].as_ref().unwrap());
        return Ok(Token::Dynamic(level.unwrap()));
    });

    lex_rule!(&literal, scale_degree_re, |cap: Vec<Option<String>>|
    {
        let d : i32 = get_nth_capture(&cap, 1)?.parse().context("Bad regex")?;
        return Ok(Token::ScaleDegree(d));
    });

    lex_rule!(&literal, measure_bar_re, |cap: Vec<Option<String>>|
    {
        return Ok(Token::MeasureBar());
    });

    lex_rule!(&literal, rest_decl_re, |cap: Vec<Option<String>>|
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

        return Ok(Token::Note(n));
    });

    bail!("No rule to lex symbol `{}`", &literal);
}

pub fn to_moonbase_str(mbn: &MoonbaseNote) -> String
{
    // the TTS engine adds about 4 seconds worth of audio for every 60
    // notes, regardless of BPM; 4000 ms / 60 notes ~= 67 ms per note.
    // however this doesn't apply to rests.

    let bias = 67;
    let mut ms = mbn.dur_ms;
    if mbn.prefix != "_" && mbn.dur_ms > bias
    {
        ms -= bias
    }

    let mut prefix : &str = &mbn.prefix;
    if prefix == "."
    {
        prefix = "duh";
    }
    if prefix == "the" // maybe will add more common words
    {
        prefix = "thuh";
    }
    if prefix == "o"
    {
        prefix = "ow";
    }
    if prefix == "a"
    {
        prefix = "ey";
    }
    if prefix == "and"
    {
        prefix = "ey-nd";
    }
    if prefix == "you"
    {
        prefix = "yu";
    }
    if prefix == "it"
    {
        prefix = "ih-t";
    }

    return format!("[{}<{},{}>{}]", prefix, ms, mbn.tone_id, mbn.suffix);
}

fn generate_moonbase(moonbase: &str, outpath: &str) -> Result<()>
{
    println!("{}", &moonbase);
    let path = Path::new(outpath);
    let url = format!("http://tts.cyzon.us/tts?text={}", moonbase);
    let bytes = reqwest::blocking::get(url)?.bytes()?;
    let mut file = File::create(&path)?;
    use std::io::Write;
    file.write_all(&bytes)?;
    return Ok(());
}

#[test]
fn moonbase_gen()
{
    let r1 = generate_moonbase("[duw<500,19>] [duw<500,19>]", "/tmp/result.wav");
    assert!(r1.is_ok());
    let r2 = generate_moonbase("wefwefw", "/a/e/bvwefiqd/.qwee");
    assert!(r2.is_err());
}

#[test]
fn moonbase_strings()
{
    assert_eq!("[duw<40,19>]", to_moonbase_str(&MoonbaseNote
    {
        prefix: "duw".to_string(),
        suffix: "".to_string(),
        dur_ms: 40,
        tone_id: 19
    }));

    assert_eq!("[du<53,10>th]", to_moonbase_str(&MoonbaseNote
    {
        prefix: "du".to_string(),
        suffix: "th".to_string(),
        dur_ms: 120,
        tone_id: 10
    }));

    assert_eq!("[uh<26,28>wf]", to_moonbase_str(&MoonbaseNote
    {
        prefix: "uh".to_string(),
        suffix: "wf".to_string(),
        dur_ms: 93,
        tone_id: 28
    }));
}

pub fn lex_file(inpath: &str) -> Result<Vec<Token>>
{
    let literals = read_literals_from_file(&inpath)?;
    let mut ret = vec![];
    for lit in literals
    {
        let token = lex_literal(&lit.literal)?;
        ret.push(token);
    }
    return Ok(ret);
}

pub fn beats_to_millis(beats: &Fraction, bpm: u16) -> Option<i32>
{
    return Some((beats.to_f64()? * 60000.0 / bpm as f64) as i32);
}

pub fn parse_file(tokens: &Vec<Token>) -> Result<Vec<MoonbaseNote>>
{
    let mut current_bpm : u16 = 120;
    let mut current_track = 0;
    let mut current_pitch = pitch_string_to_id("C2")?;

    let mut ret = vec![];

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
                ret.push(mb);
            }
            Token::Repeat() => (),
            Token::BeatAssert(b) => (),
            Token::Scale(s) => (),
            Token::ScaleDegree(d) => (),
            Token::Dynamic(l) => (),
            Token::MeasureBar() => ()
        }
    }

    return Ok(ret);
}

pub fn compile(inpath: &str, outpath: &str) -> Result<()>
{
    println!("{} -> {}", &inpath, &outpath);

    let tokens = lex_file(inpath)?;
    let parsed = parse_file(&tokens)?;
    let mb = parsed.iter().map(|m| to_moonbase_str(m))
        .collect::<Vec<String>>().join("");
    generate_moonbase(&mb, outpath)?;

    return Ok(());
}

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
    lex_assert!(":|", Token::Repeat());
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
fn dynamic_lexing()
{
    lex_assert!("PIANISSIMO", Token::Dynamic(DynamicLevel::PIANISSIMO));
    lex_assert!("PIANO",      Token::Dynamic(DynamicLevel::PIANO));
    lex_assert!("MEZZOPIANO", Token::Dynamic(DynamicLevel::MEZZOPIANO));
    lex_assert!("MEZZOFORTE", Token::Dynamic(DynamicLevel::MEZZOFORTE));
    lex_assert!("FORTE",      Token::Dynamic(DynamicLevel::FORTE));
    lex_assert!("FORTISSIMO", Token::Dynamic(DynamicLevel::FORTISSIMO));
}

#[test]
fn garbage_lexing()
{
    assert!(lex_literal("wefwe$234").is_err());
    assert!(lex_literal("dddFd").is_err());
    assert!(lex_literal("...--").is_err());
}

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
    assert_result!(compile("examples/batman.reg",     "/tmp/batman.mp3"),     ());
    assert_result!(compile("examples/campfire.reg",   "/tmp/campfire.mp3"),   ());
    assert_result!(compile("examples/choir_test.reg", "/tmp/choir_test.mp3"), ());
    assert_result!(compile("examples/dynamics.reg",   "/tmp/dynamics.mp3"),   ());
    assert_result!(compile("examples/hbjm.reg",       "/tmp/hbjm.mp3"),       ());
    assert_result!(compile("examples/regularity.reg", "/tmp/regularity.mp3"), ());
    assert_result!(compile("examples/scales.reg",     "/tmp/scales.mp3"),     ());
    assert_result!(compile("examples/mariah.reg",     "/tmp/mariah.mp3"),     ());

    assert_result!(compile(
        "examples/thelionsleepstonight.reg",
        "/tmp/thelionsleepstonight.mp3"), ());
}