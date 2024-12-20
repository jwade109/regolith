use std::fs::read_to_string;
use fraction::Fraction;
use regex_macro::regex;
use crate::types::*;
use std::path::Path;

use crate::moonbase::MoonbaseNote;

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

pub fn pitch_string_to_id(pitch: &str) -> Option<ToneId>
{
    let (_, i) = PITCH_MAP.iter().find(|(s, _)| *s == pitch)?;
    Some(ToneId(*i))
}

#[test]
fn pitch_string_conversions()
{
    assert_eq!(pitch_string_to_id("C1"),  Some(ToneId(1)));
    assert_eq!(pitch_string_to_id("D2#"), Some(ToneId(16)));
    assert_eq!(pitch_string_to_id("A2#"), Some(ToneId(23)));
    assert_eq!(pitch_string_to_id("G3"),  Some(ToneId(32)));
    assert_eq!(pitch_string_to_id("C4"),  Some(ToneId(37)));
    assert_eq!(pitch_string_to_id(""),    None);
    assert_eq!(pitch_string_to_id("J3"),  None);
    assert_eq!(pitch_string_to_id("Bb"),  None);
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

pub fn read_literals_from_multiline_string(source: &str, filename: &str) -> CompileResult<Vec<Literal>>
{
    let mut result = Vec::new();
    let mut idno = 0;

    let reg = regex!(r"[^\s]+");

    for (lineno, line) in source.lines().enumerate()
    {
        if line.is_empty() || line.starts_with('#')
        {
            continue;
        }

        for m in reg.find_iter(&line)
        {
            let l = Literal
            {
                colno: m.start(),
                filename: filename.to_string(),
                lineno,
                literal: m.as_str().to_string(),
                idno
            };
            idno += 1;
            result.push(l);
        }

        result.push(Literal
        {
            colno: line.len(),
            filename: filename.to_string(),
            lineno,
            literal: "<eol>".to_string(),
            idno
        });
        idno += 1;
    }

    Ok(result)
}

fn read_literals_from_file(filename: &Path) -> CompileResult<Vec<Literal>>
{
    read_literals_from_multiline_string(&read_to_string(filename)
        .or(Err(CompileError::Generic("Failed to open file".to_string())))?,
        filename.to_str().ok_or(CompileError::Generic("Bad path".to_string()))?)
}

pub fn read_literals_from_markdown(filename: &Path) -> CompileResult<Vec<Literal>>
{
    let mut result = Vec::new();
    let mut idno = 0;

    let reg = regex!(r"[^\s]+");

    let mut codeblock = false;

    let name = filename.to_str().ok_or(
        CompileError::Generic("Bad filename".to_string()))?.to_string();

    for (lineno, line) in read_to_string(filename)
        .or(Err(CompileError::Generic("Failed to open file".to_string())))?.lines().enumerate()
    {
        if line.is_empty() || line.starts_with('#')
        {
            continue;
        }

        if line == "```regolith"
        {
            codeblock = true;
            continue
        }
        else if line == "```"
        {
            codeblock = false;
            continue;
        }

        if !codeblock
        {
            continue;
        }

        for m in reg.find_iter(&line)
        {
            let l = Literal
            {
                colno: m.start() + 1,
                filename: name.clone(),
                lineno: lineno + 1,
                literal: m.as_str().to_string(),
                idno
            };
            idno += 1;
            result.push(l);
        }

        result.push(Literal
        {
            colno: line.len() + 1,
            filename: name.clone(),
            lineno: lineno + 1,
            literal: "<eol>".to_string(),
            idno
        });
        idno += 1;
    }

    Ok(result)
}

macro_rules! lex_rule
{
    ($lit: expr, $re: expr, $callable: expr) =>
    {
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
            Some(result) => assert_eq!(result, $expect),
            None =>
            {
                assert!(false);
            }
        }
    }
}

#[allow(unused_macros)]
macro_rules! lex_nope
{
    ($string: expr) =>
    {
        assert_eq!(lex_literal($string), None);
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

fn get_nth_capture(captures: &[Option<String>], i: usize) -> Option<String>
{
    captures.get(i)?.clone()
}

fn lex_literal(literal: &str) -> Option<Token>
{
    if literal == "<eol>"
    {
        return Some(Token::Endline());
    }

    let measure_bar_re = regex!(r"^(:?)\|(:?)$");
    let bpm_token_re = regex!(r"^(\d+)BPM$");
    let track_token_re = regex!(r"^\[(\d+)\]$");
    let pitch_token_re = regex!(r"^[A-Z]\d?#?$");
    let scale_degree_re = regex!(r"^(\d+)([#b])?$");
    let note_token_re = regex!(r"^([a-z\.]+)\-?([a-z\.]+)?(:(\d+))?(\/(\d+))?$");
    let scale_decl_re = regex!(r"^([A-G]\d*[#b]?)(\[(\d+)\]|PENTA|MAJOR|MINOR|CHROM)?$");
    let dynamic_decl_re = regex!(r"^FORTISSIMO|FORTE|MEZZOFORTE|MEZZOPIANO|PIANO|PIANISSIMO$");
    let rest_decl_re = regex!(r"^-(:(\d+))?(\/(\d+))?$");
    let section_marker_re = regex!(r"^===([^\s-]*)===$");
    let time_signature_re = regex!(r"^(\d+)\/(\d+)$");

    lex_rule!(&literal, bpm_token_re, |cap: &[Option<String>]|
    {
        let bpm : u16 = get_nth_capture(cap, 1)?.parse().ok()?;
        Some(Token::Tempo(bpm))
    });

    lex_rule!(&literal, track_token_re, |cap: &[Option<String>]|
    {
        let track_id : u32 = get_nth_capture(cap, 1)?.parse().ok()?;
        Some(Token::Track(track_id))
    });

    lex_rule!(&literal, pitch_token_re, |cap: &[Option<String>]|
    {
        let s : String = get_nth_capture(cap, 0)?;
        let id = pitch_string_to_id(&s)?;
        Some(Token::AbsolutePitch(id))
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
        Some(Token::Note(n))
    });

    lex_rule!(&literal, scale_decl_re, |cap: &[Option<String>]|
    {
        let pitch_str = get_nth_capture(cap, 1)?;
        let tone_id = pitch_string_to_id(&pitch_str)?;
        let steps : Vec<u8> = if let Some(numbers) = cap.get(3)?
        {
            numbers.chars().map(|c| c.to_digit(10).unwrap() as u8).collect::<Vec<_>>()
        }
        else
        {
            get_named_scale_steps(&get_nth_capture(cap, 2)?)?
        };

        let s = Scale
        {
            name: literal.to_string(),
            tone_id,
            steps
        };

        Some(Token::Scale(s))
    });

    lex_rule!(&literal, dynamic_decl_re, |cap: &[Option<String>]|
    {
        let level = parse_dynamic_level(cap[0].as_ref().unwrap());
        Some(Token::Dynamic(level.unwrap()))
    });

    lex_rule!(&literal, scale_degree_re, |cap: &[Option<String>]|
    {
        let d : u8 = get_nth_capture(cap, 1)?.parse().ok()?;
        Some(Token::ScaleDegree(d))
    });

    lex_rule!(&literal, measure_bar_re, |cap: &[Option<String>]|
    {
        let prefix = get_nth_capture(cap, 1)?;
        let suffix = get_nth_capture(cap, 2)?;
        Some(Token::MeasureBar(prefix == ":", suffix == ":"))
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

        Some(Token::Note(n))
    });

    lex_rule!(&literal, section_marker_re, |cap: &[Option<String>]|
    {
        let name = get_nth_capture(cap, 1)?;
        Some(Token::Section(name))
    });

    lex_rule!(&literal, time_signature_re, |cap: &[Option<String>]|
    {
        let numer : u8 = get_nth_capture(cap, 1)?.parse().ok()?;
        let denom : u8 = get_nth_capture(cap, 2)?.parse().ok()?;
        Some(Token::TimeSignature((numer, denom)))
    });

    None
}

pub fn lex_literals(literals: &Vec<Literal>) -> CompileResult<Vec<(Literal, Token)>>
{
    let mut ret = vec![];
    for lit in literals
    {
        let token = lex_literal(&lit.literal)
            .ok_or(CompileError::InvalidSyntax(lit.clone()))?;
        ret.push((lit.clone(), token));
    }
    Ok(ret)
}

pub fn lex_multiline_string(source: &str) -> CompileResult<Vec<(Literal, Token)>>
{
    lex_literals(&read_literals_from_multiline_string(source, "")?)
}

pub fn lex_file(inpath: &Path) -> CompileResult<Vec<(Literal, Token)>>
{
    lex_literals(&read_literals_from_file(inpath)?)
}

pub fn lex_markdown(inpath: &Path) -> CompileResult<Vec<(Literal, Token)>>
{
    lex_literals(&read_literals_from_markdown(inpath)?)
}

// TODO
// fn beats_to_millis(beats: &Fraction, bpm: u16) -> Option<i32>
// {
//     Some((beats.to_f64()? * 60000.0 / bpm as f64) as i32)
// }

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
    lex_assert!("C", Token::AbsolutePitch(ToneId(13)));
    lex_assert!("D", Token::AbsolutePitch(ToneId(15)));
    lex_assert!("E", Token::AbsolutePitch(ToneId(17)));
}

#[test]
fn relative_pitch_lexing()
{
    lex_assert!("1", Token::ScaleDegree(1));
    lex_assert!("2", Token::ScaleDegree(2));
    lex_assert!("5", Token::ScaleDegree(5));
    lex_assert!("13", Token::ScaleDegree(13));

    lex_nope!("-4");
    lex_nope!("352d");
}

#[test]
fn scale_lexing()
{
    lex_assert!("CMAJOR", Token::Scale(Scale
    {
        name: "CMAJOR".to_string(),
        tone_id: ToneId(13),
        steps: vec![2, 2, 1, 2, 2, 2, 1]
    }));

    lex_assert!("AMINOR", Token::Scale(Scale
    {
        name: "AMINOR".to_string(),
        tone_id: ToneId(10),
        steps: vec![2, 1, 2, 2, 1, 2, 2]
    }));

    lex_assert!("G#PENTA", Token::Scale(Scale
    {
        name: "G#PENTA".to_string(),
        tone_id: ToneId(21),
        steps: vec![2, 2, 3, 2, 3]
    }));

    lex_assert!("D3#CHROM", Token::Scale(Scale
    {
        name: "D3#CHROM".to_string(),
        tone_id: ToneId(28),
        steps: vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
    }));

    // no brackets
    lex_nope!("Fb3");

    // bad pitch
    lex_nope!("K4[22211]");
}

#[test]
fn bar_lexing()
{
    lex_assert!("|",  Token::MeasureBar(false, false));
    lex_assert!("|:", Token::MeasureBar(false, true));
    lex_assert!(":|", Token::MeasureBar(true, false));
    lex_assert!(":|:", Token::MeasureBar(true, true));
    // lex_assert!(":|x2",  Token::EndRepeat(2));
    // lex_assert!(":|x6",  Token::EndRepeat(6));
    // lex_assert!(":|x12", Token::EndRepeat(12));
    // lex_assert!(":|x0",  Token::EndRepeat(0));

    lex_nope!(":|x-1");
    lex_nope!(":|x-5");
    lex_nope!(":|x-15");
}

#[test]
fn bpm_lexing()
{
    lex_assert!("120BPM",  Token::Tempo(120));
    lex_assert!("92BPM",   Token::Tempo(92));
    lex_assert!("1103BPM", Token::Tempo(1103));
    lex_assert!("0BPM",    Token::Tempo(0));

    lex_nope!("-12BPM");
    lex_nope!("CHEESEBPM");
    lex_nope!("--BPM");
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
    lex_nope!("wefwe$234");
    lex_nope!("dddFd");
    lex_nope!("...--");
}

#[test]
fn section_lexing()
{
    lex_assert!("======",      Token::Section("".to_string()));
    lex_assert!("===hello===", Token::Section("hello".to_string()));
    lex_assert!("===GOO===",   Token::Section("GOO".to_string()));
    lex_assert!("===34g===",   Token::Section("34g".to_string()));
}
