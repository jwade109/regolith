#[allow(warnings)]

use crate::types::{CompileError, CompileResult, ToneId};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::io::Write;

#[derive(Debug)]
pub struct MoonbaseNote
{
    pub prefix: String,
    pub suffix: String,
    pub dur_ms: i32,
    pub tone_id: ToneId
}

fn hashed_fn(arg: &str, ext: &str, tmp_dir: &Path) -> PathBuf
{
    let digest = md5::compute(arg);
    PathBuf::from(&tmp_dir.join(format!("{:x}.{}", digest, ext)))
}

#[test]
fn filename_hashing()
{
    assert_eq!(
        hashed_fn("ewjwef", "wav", Path::new("/tmp/")),
        Path::new("/tmp/fc0d3155c1b5099b40038d39cc71963e.wav")
    );

    assert_eq!(
        hashed_fn("", "jpg", Path::new("/tmp/")),
        Path::new("/tmp/d41d8cd98f00b204e9800998ecf8427e.jpg")
    );

    assert_eq!(
        hashed_fn("[duw<40,19>]", "mp3", Path::new("/tmp/")),
        Path::new("/tmp/a85cb3b84d6813ab169ddca8a03be747.mp3")
    );
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

    let ToneId(t) = mbn.tone_id;
    format!("[{}<{},{}>{}]", prefix, ms, t, mbn.suffix)
}

#[test]
fn moonbase_strings()
{
    assert_eq!("[duw<40,19>]", to_moonbase_str(&MoonbaseNote
    {
        prefix: "duw".to_string(),
        suffix: "".to_string(),
        dur_ms: 40,
        tone_id: ToneId(19)
    }));

    assert_eq!("[du<53,10>th]", to_moonbase_str(&MoonbaseNote
    {
        prefix: "du".to_string(),
        suffix: "th".to_string(),
        dur_ms: 120,
        tone_id: ToneId(10)
    }));

    assert_eq!("[uh<26,28>wf]", to_moonbase_str(&MoonbaseNote
    {
        prefix: "uh".to_string(),
        suffix: "wf".to_string(),
        dur_ms: 93,
        tone_id: ToneId(28)
    }));
}

pub fn create_dir(p: &Path) -> Result<(), std::io::Error>
{
    if !p.exists()
    {
        std::fs::create_dir(p)
    }
    else
    {
        Ok(())
    }
}

pub fn generate_moonbase(moonbase: &str, tmp_dir: &Path) -> CompileResult<PathBuf>
{
    // TODO
    // let num_attempts = 10;
    // let backoff_dur = Duration::new(2, 0);

    let outpath = hashed_fn(moonbase, "wav", tmp_dir);
    if outpath.exists()
    {
        return Ok(outpath);
    }

    let url = format!("http://tts.cyzon.us/tts?text={}", moonbase);

    let resp = reqwest::blocking::get(&url)?;
    resp.error_for_status_ref()?;
    create_dir(tmp_dir)?;
    let mut file = File::create(&outpath)?;
    let bytes = resp.bytes()?;
    file.write_all(&bytes)?;
    return Ok(outpath);
}

#[test]
fn moonbase_gen()
{
    let tmp_dir = Path::new("/tmp/");

    assert_eq!(
        generate_moonbase("[duw<500,19>] [duw<500,19>]", tmp_dir).unwrap(),
        Path::new("/tmp/regolith/0f4ed7068d8362b1c2dafa2baea51b5d.wav")
    );

    assert_eq!(
        generate_moonbase("wefwefw", tmp_dir).unwrap(),
        Path::new("/tmp/37e838885e9fd07692e5da83e515878e.wav")
    );

    assert_eq!(
        generate_moonbase("command error in phoneme", tmp_dir).unwrap(),
        Path::new("/tmp/b1ec37d0fe49d4b46bb7f1ad801ae335.wav")
    );

    assert_eq!(
        generate_moonbase("[duw<500,19>] [duw<500,19>] command error in phoneme", tmp_dir).unwrap(),
        Path::new("/tmp/834abde08a1c2303efd64755f2ad84fb.wav")
    );
}
