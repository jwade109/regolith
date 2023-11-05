#![allow(dead_code, unused)]

use fraction::Fraction;
use std::fs::read_to_string;
use regex::Regex;
use regex_macro::regex;
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::error::Error;
use anyhow::{Result, Context, bail};

extern crate reqwest;

#[derive(Debug, PartialEq)]
pub struct Literal
{
    literal: String,
    filename: String,
    lineno: usize,
    colno: i32,
}

#[derive(Debug, PartialEq)]
pub struct TrackDirective
{
    track_id: i32
}

#[derive(Debug, PartialEq)]
pub struct TempoDirective
{
    bpm: i32
}

#[derive(Debug, PartialEq)]
pub struct PitchDirective
{
    tone_str: String
}

#[derive(Debug, PartialEq)]
pub struct RegoNote
{
    prefix: String,
    suffix: String,
    beats: Fraction
}

#[derive(Debug, PartialEq)]
pub struct MoonbaseNote
{
    prefix: String,
    suffix: String,
    dur_ms: i32,
    tone_id: i32
}

#[derive(Debug, PartialEq)]
pub enum DynamicLevel
{
    PIANISSIMO,
    PIANO,
    MEZZOPIANO,
    MEZZOFORTE,
    FORTE,
    FORTISSIMO
}

#[derive(Debug, PartialEq)]
pub enum Token
{
    Track(TrackDirective),
    Tempo(TempoDirective),
    Pitch(PitchDirective),
    Note(RegoNote),
    Repeat(),
    BeatAssert(i32),
    Scale(),
    ScaleDegree(),
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

macro_rules! parse_rule
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

pub fn tokenize_literal(literal: &str) -> Result<Token>
{
    let measure_bar_re = regex!(r"^\|");
    let repeat_token_re = regex!(r"^\:\|");
    let beat_assert_re = regex!(r"^@(\d+)$");
    let bpm_token_re = regex!(r"^(\d+)BPM$");
    let track_token_re = regex!(r"^TRACK(\d+)$");
    let pitch_token_re = regex!(r"^[A-G]\d?#?$");
    let scale_degree_re = regex!(r"^(\d+)([#b])?$");
    let note_token_re = regex!(r"^([a-z\.]+)\-?([a-z\.]+)?(:(\d+))?(\/(\d+))?$");
    let scale_decl_re = regex!(r"^([A-G]\d*[#b]?)\[?((\d+)|PENTA|MAJOR|MINOR|CHROM)\]?$");
    let dynamic_decl_re = regex!(r"^FORTISSIMO|FORTE|MEZZOFORTE|MEZZOPIANO|PIANO|PIANISSIMO");
    let rest_decl_re = regex!(r"^-(:(\d+))?(\/(\d+))?$");

    parse_rule!(&literal, bpm_token_re, |cap: Vec<Option<String>>|
    {
        let bpm : i32 = cap.get(1).context("Bad regex")?
                         .as_ref().context("Bad regex")?
                          .parse().context("Bad regex")?;
        return Ok(Token::Tempo(TempoDirective{bpm: bpm}))
    });

    parse_rule!(&literal, track_token_re, |cap: Vec<Option<String>>|
    {
        let idx : i32 = cap.get(1).context("Bad regex")?
                         .as_ref().context("Bad regex")?
                          .parse().context("Bad regex")?;
        return Ok(Token::Track(TrackDirective{track_id: idx}))
    });

    parse_rule!(&literal, pitch_token_re, |cap: Vec<Option<String>>|
    {
        let s = cap.get(0).context("Bad regex")?
                 .as_ref().context("Bad regex")?.clone();
        return Ok(Token::Pitch(PitchDirective{tone_str: s}));
    });

    parse_rule!(&literal, note_token_re, |cap: Vec<Option<String>>|
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

    parse_rule!(&literal, repeat_token_re, |cap: Vec<Option<String>>|
    {
        return Ok(Token::Repeat());
    });

    parse_rule!(&literal, beat_assert_re, |cap: Vec<Option<String>>|
    {
        let beats : i32 = cap[1].as_ref().unwrap().parse().unwrap();
        return Ok(Token::BeatAssert(beats));
    });

    parse_rule!(&literal, scale_decl_re, |cap: Vec<Option<String>>|
    {
        return Ok(Token::Scale());
    });

    parse_rule!(&literal, dynamic_decl_re, |cap: Vec<Option<String>>|
    {
        let level = parse_dynamic_level(cap[0].as_ref().unwrap());
        return Ok(Token::Dynamic(level.unwrap()));
    });

    parse_rule!(&literal, scale_degree_re, |cap: Vec<Option<String>>|
    {
        return Ok(Token::ScaleDegree());
    });

    parse_rule!(&literal, measure_bar_re, |cap: Vec<Option<String>>|
    {
        return Ok(Token::MeasureBar());
    });

    parse_rule!(&literal, rest_decl_re, |cap: Vec<Option<String>>|
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
            prefix: "".to_string(),
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

    return format!("[{}<{},{}>{}]", mbn.prefix, ms, mbn.tone_id, mbn.suffix);
}

fn generate_moonbase(moonbase: &str, path: &Path) -> Result<()>
{
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
    let r1 = generate_moonbase("[duw<500,19>] [duw<500,19>]", &Path::new("/tmp/result.wav"));
    assert!(r1.is_ok());
    let r2 = generate_moonbase("wefwefw", &Path::new("/a/e/bvwefiqd/.qwee"));
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

pub fn read_tokens_from_file(inpath: &str) -> Result<Vec<Token>>
{
    let literals = read_literals_from_file(&inpath)?;
    let mut ret = vec![];
    for lit in literals
    {
        let token = tokenize_literal(&lit.literal)?;
        ret.push(token);
    }
    return Ok(ret);
}

pub fn compile(inpath: &str, outpath: &str) -> Result<()>
{
    println!("{} -> {}", &inpath, &outpath);

    let tokens = read_tokens_from_file(inpath)?;

    for t in tokens
    {
        if let Token::Note(ref n) = t
        {
            let m = MoonbaseNote
            {
                prefix: n.prefix.clone(),
                suffix: n.suffix.clone(),
                tone_id: 14,
                dur_ms: 250
            };
            let mbs = to_moonbase_str(&m);
            // println!("{:?}, {}", t, mbs);
        }
    }

    return Ok(());
}

#[test]
fn note_parsing()
{
    assert_eq!(tokenize_literal("ih-s:3/2").unwrap(),
    Token::Note(RegoNote
    {
        prefix: "ih".to_string(),
        suffix: "s".to_string(),
        beats: Fraction::new(3u64, 2u64)
    }));

    assert_eq!(tokenize_literal("uh-n/2").unwrap(),
    Token::Note(RegoNote
    {
        prefix: "uh".to_string(),
        suffix: "n".to_string(),
        beats: Fraction::new(1u64, 2u64)
    }));

    assert_eq!(tokenize_literal("ne/3").unwrap(),
    Token::Note(RegoNote
    {
        prefix: "ne".to_string(),
        suffix: "".to_string(),
        beats: Fraction::new(1u64, 3u64)
    }));

    assert_eq!(tokenize_literal("-:12").unwrap(),
    Token::Note(RegoNote
    {
        prefix: "".to_string(),
        suffix: "".to_string(),
        beats: Fraction::new(12u64, 1u64)
    }));
}

#[test]
fn scale_parsing()
{
    // TODO flesh this out
    assert_eq!(tokenize_literal("CMAJOR").unwrap(),  Token::Scale());
    assert_eq!(tokenize_literal("AMINOR").unwrap(),  Token::Scale());
    assert_eq!(tokenize_literal("DbMINOR").unwrap(), Token::Scale());
    assert_eq!(tokenize_literal("G#PENTA").unwrap(), Token::Scale());
}

#[test]
fn bar_parsing()
{
    assert_eq!(tokenize_literal("|").unwrap(), Token::MeasureBar());
}

#[test]
fn repeat_parsing()
{
    assert_eq!(tokenize_literal(":|").unwrap(), Token::Repeat());
}

#[test]
fn beats_assert_parsing()
{
    assert_eq!(tokenize_literal("@16").unwrap(),   Token::BeatAssert(16));
    assert_eq!(tokenize_literal("@32").unwrap(),   Token::BeatAssert(32));
    assert_eq!(tokenize_literal("@27").unwrap(),   Token::BeatAssert(27));
    assert_eq!(tokenize_literal("@0").unwrap(),    Token::BeatAssert(0));
    assert_eq!(tokenize_literal("@2452").unwrap(), Token::BeatAssert(2452));
    assert!(   tokenize_literal("@-3").is_err());
}

#[test]
fn bpm_parsing()
{
    assert_eq!(tokenize_literal("120BPM").unwrap(),
    Token::Tempo(TempoDirective
    {
        bpm: 120
    }));
    assert_eq!(tokenize_literal("92BPM").unwrap(),
    Token::Tempo(TempoDirective
    {
        bpm: 92
    }));
    assert_eq!(tokenize_literal("1103BPM").unwrap(),
    Token::Tempo(TempoDirective
    {
        bpm: 1103
    }));
    assert_eq!(tokenize_literal("0BPM").unwrap(),
    Token::Tempo(TempoDirective
    {
        bpm: 0
    }));

    assert!(tokenize_literal("-12BPM").is_err());
    assert!(tokenize_literal("CHEESEBPM").is_err());
    assert!(tokenize_literal("--BPM").is_err());
}

#[test]
fn dynamic_parsing()
{
    assert_eq!(tokenize_literal("PIANISSIMO").unwrap(), Token::Dynamic(DynamicLevel::PIANISSIMO));
    assert_eq!(tokenize_literal("PIANO").unwrap(),      Token::Dynamic(DynamicLevel::PIANO));
    assert_eq!(tokenize_literal("MEZZOPIANO").unwrap(), Token::Dynamic(DynamicLevel::MEZZOPIANO));
    assert_eq!(tokenize_literal("MEZZOFORTE").unwrap(), Token::Dynamic(DynamicLevel::MEZZOFORTE));
    assert_eq!(tokenize_literal("FORTE").unwrap(),      Token::Dynamic(DynamicLevel::FORTE));
    assert_eq!(tokenize_literal("FORTISSIMO").unwrap(), Token::Dynamic(DynamicLevel::FORTISSIMO));
}

#[test]
fn garbage_parsing()
{
    assert!(tokenize_literal("wefwe$234").is_err());
    assert!(tokenize_literal("dddFd").is_err());
    assert!(tokenize_literal("...--").is_err());
}
