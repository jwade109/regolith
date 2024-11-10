use std::collections::HashSet;

use crate::semantics::Composition;
use crate::types::{NoteDecl, CompileResult};
use crate::moonbase::{generate_moonbase, to_moonbase_str, MoonbaseNote};
use fraction::{Fraction, ToPrimitive};
use std::path::Path;

fn bpm_to_millis(beats: &Fraction, bpm: u16) -> i32
{
    let milliseconds_per_beat = Fraction::new(60000 as u64, bpm);
    (beats * milliseconds_per_beat).to_i32().unwrap_or(0)
}

fn to_moonbase_note(bpm: u16, n: &NoteDecl) -> MoonbaseNote
{
    MoonbaseNote
    {
        prefix: n.note.prefix.clone(),
        suffix: n.note.suffix.clone(),
        dur_ms: bpm_to_millis(&n.note.beats, bpm),
        tone_id: n.tone_id
    }
}

pub fn generate_mb_code(comp: &Composition, build_dir: &Path) -> CompileResult<()>
{
    for section in &comp.sections
    {
        let tracks = section.measures.iter().map(|m|
        {
            &m.track
        }).collect::<HashSet<_>>();

        for tid in tracks
        {
            let sec: String = section.measures.iter().filter(|m| {
                m.track == *tid
            })
            .map(|m| {
                m.notes.iter().map(|n| {
                    to_moonbase_str(&to_moonbase_note(160, n))
                }).collect::<String>()
            }).collect();

            let res = generate_moonbase(&sec)?;
            let dst: std::path::PathBuf = build_dir.join(format!(
                "section-{}-track-{}.wav", section.id, tid
            ));
            std::fs::copy(&res, &dst);
            println!("{}", dst.display());
        }
    }

    Ok(())
}
