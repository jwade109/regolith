use anyhow::Result;
use std::fs::File;
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub struct MoonbaseNote
{
    pub prefix: String,
    pub suffix: String,
    pub dur_ms: i32,
    pub tone_id: u8
}

fn hashed_fn(arg: &str, ext: &str) -> Result<String>
{
    let digest = md5::compute(arg);
    Ok(format!("/tmp/{:x}.{}", digest, ext))
}

#[test]
fn moonbase_string_hashing()
{
    assert_eq!(
        hashed_fn("ewjwef", "wav").ok(),
        Some("/tmp/fc0d3155c1b5099b40038d39cc71963e.wav".to_string())
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

    format!("[{}<{},{}>{}]", prefix, ms, mbn.tone_id, mbn.suffix)
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

pub fn generate_moonbase(moonbase: &str) -> Result<String>
{
    let mut outpath = hashed_fn(&moonbase, "wav")?;
    let path = Path::new(&outpath);
    if !path.exists()
    {
        let mut file = File::create(&path)?;
        let url = format!("http://tts.cyzon.us/tts?text={}", moonbase);
        let resp = reqwest::blocking::get(url)?;
        resp.error_for_status_ref()?;
        use std::io::Write;
        file.write_all(&resp.bytes()?)?;
    }
    Ok(outpath)
}

#[test]
fn moonbase_gen()
{
    assert_eq!(
        generate_moonbase("[duw<500,19>] [duw<500,19>]").ok(),
        Some("/tmp/0f4ed7068d8362b1c2dafa2baea51b5d.wav".to_string())
    );

    assert_eq!(
        generate_moonbase("wefwefw").ok(),
        Some("/tmp/37e838885e9fd07692e5da83e515878e.wav".to_string())
    );
}
