use anyhow::Result;

use crate::lexer::{lex_file, parse_tokens};
use crate::moonbase::{to_moonbase_str, MoonbaseNote, generate_moonbase};

#[derive(Debug)]
pub struct Sequence
{
    pub id: u8,
    pub notes: Vec<MoonbaseNote> // TODO public?
}

fn generate_from_sequences(sequences: &Vec<Sequence>) -> Result<()>
{
    for seq in sequences
    {
        let mb = seq.notes.iter().map(to_moonbase_str)
            .collect::<Vec<String>>().join("");
        generate_moonbase(&mb)?;
    }
    Ok(())
}

pub fn compile(inpath: &str, outpath: &str) -> Result<()>
{
    println!("{} -> {}", &inpath, &outpath);
    let tokens = lex_file(inpath)?;
    let sequences = parse_tokens(&tokens)?;
    generate_from_sequences(&sequences)?;
    Ok(())
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
    std::fs::create_dir("rust_songs");
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
