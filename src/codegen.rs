use std::collections::HashSet;

use reqwest::StatusCode;
use crate::semantics::Composition;
use crate::types::{CompileError, CompileResult, NoteDecl};
use crate::moonbase::{create_dir, generate_moonbase, to_moonbase_str, MoonbaseError, MoonbaseNote};
use fraction::{Fraction, ToPrimitive};
use std::path::Path;

fn bpm_to_millis(beats: &Fraction, bpm: u16) -> Option<i32>
{
    let milliseconds_per_beat = Fraction::new(60000 as u64, bpm);
    Some((beats * milliseconds_per_beat).to_f64()?.round() as i32)
}

fn to_moonbase_note(bpm: u16, n: &NoteDecl) -> MoonbaseNote
{
    MoonbaseNote
    {
        prefix: n.note.prefix.clone(),
        suffix: n.note.suffix.clone(),
        dur_ms: bpm_to_millis(&n.note.beats, bpm).unwrap_or(0),
        tone_id: n.tone_id
    }
}

pub fn generate_mb_code(comp: &Composition, cache_dir: &Path, build_dir: &Path) -> CompileResult<()>
{
    let text_dir = build_dir.join("mb_text");
    create_dir(&text_dir)?;

    for section in &comp.sections
    {
        for (track_id, measures) in &section.tracks
        {
            let sec: String = measures.iter().map(|m| {
                m.notes.iter().map(|n: &NoteDecl| {
                    to_moonbase_str(&to_moonbase_note(section.tempo, n))
                }).collect::<String>()
            }).collect();

            let mb_txt_path = text_dir.join(
                format!("section-{}-track-{}.txt", section.id, track_id));

            if mb_txt_path.exists()
            {
                std::fs::remove_file(&mb_txt_path)?;
            }
            std::fs::write(mb_txt_path, &sec)?;

            let res = match generate_moonbase(&sec, cache_dir)
            {
                Ok(path) => path,
                Err(e) => match e
                {
                    MoonbaseError::FileError(fe) => return Err(CompileError::FileError(fe)),
                    MoonbaseError::NetworkError(ne) =>
                    {
                        match ne.status()
                        {
                            Some(status) =>
                            {
                                match status
                                {
                                    StatusCode::TOO_MANY_REQUESTS => return Err(CompileError::TooManyRequests),
                                    StatusCode::PAYLOAD_TOO_LARGE =>
                                    {

                                        return Err(CompileError::TrackTooLarge)
                                    }
                                    _ => return Err(CompileError::NetworkError(ne)),
                                }
                            }
                            None => return Err(CompileError::NetworkError(ne)),
                        }
                    }
                }
            };

            let dst: std::path::PathBuf = build_dir.join(format!(
                "section-{}-track-{}.wav", section.id, track_id
            ));
            std::fs::copy(&res, &dst);
            println!("{}", dst.display());
        }
    }

    Ok(())
}
