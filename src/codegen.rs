use std::collections::HashSet;

use hound::WavSpec;
use reqwest::StatusCode;
use crate::semantics::Composition;
use crate::types::{CompileError, CompileResult, NoteDecl};
use crate::moonbase::{create_dir, generate_moonbase, to_moonbase_str, MoonbaseError, MoonbaseNote};
use fraction::{Fraction, ToPrimitive};
use std::path::{Path, PathBuf};

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

fn generate_moonbase_or_error(moonbase: &str, tmp_dir: &Path) -> CompileResult<PathBuf>
{
    match generate_moonbase(moonbase, tmp_dir)
    {
        Ok(path) => Ok(path),
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
    }
}

fn load_samples(tracks: &[PathBuf]) -> CompileResult<(Vec<Vec<i16>>, hound::WavSpec)>
{
    let mut spec = None;
    let samples = tracks.iter().map(|p: &PathBuf|
    {
        let mut reader = hound::WavReader::open(p).unwrap();
        if spec.is_none()
        {
            spec = Some(reader.spec());
        }
        reader.samples::<i16>().collect::<Result<Vec<_>, _>>()
    })
    .collect::<Result<Vec<_>, _>>()?;

    Ok((samples, spec.unwrap()))
}

fn write_samples(samples: &Vec<i16>, out: &Path, spec: &WavSpec) -> CompileResult<()>
{
    let mut writer = hound::WavWriter::create(out, *spec)?;
    for s in samples
    {
        writer.write_sample(*s)?;
    }
    Ok(())
}

fn overlay_tracks(tracks: &[PathBuf], out: &Path) -> CompileResult<()>
{
    let (samples, spec) = load_samples(tracks)?;
    let len: usize = samples.iter().map(|s| s.len()).min().unwrap();
    let sum : Vec<i16> = (0..len).map(|i: usize| (0..tracks.len()).map(|j| samples[j][i]).sum()).collect();
    write_samples(&sum, out, &spec)?;
    Ok(())
}

fn append_tracks(tracks: &[PathBuf], out: &Path) -> CompileResult<()>
{
    let (samples, spec) = load_samples(tracks)?;
    let concat : Vec<i16> = samples.into_iter().flatten().collect();
    write_samples(&concat, out, &spec)?;
    Ok(())
}

pub fn generate_mb_code(comp: &Composition, cache_dir: &Path, build_dir: &Path) -> CompileResult<()>
{
    let text_dir = build_dir.join("mb_text");
    create_dir(&text_dir)?;

    let song_out = build_dir.join("song.wav");

    let section_wavs = comp.sections.iter().filter(|s|
    {
        !s.tracks.is_empty()
    })
    .map(|section|
    {
        let section_out = build_dir.join(format!("section-{}-output.wav", section.id));

        let trackfiles = section.tracks.iter().map(|(track_id, measures)|
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

            let res = generate_moonbase_or_error(&sec, cache_dir)?;

            let dst: std::path::PathBuf = build_dir.join(format!(
                "section-{}-track-{}.wav", section.id, track_id
            ));
            std::fs::copy(&res, &dst);

            return Ok::<PathBuf, CompileError>(res);

        })
        .collect::<CompileResult<Vec<PathBuf>>>()?;

        overlay_tracks(&trackfiles, &section_out)?;

        Ok::<PathBuf, CompileError>(section_out)
    })
    .collect::<Result<Vec<_>, _>>()?;

    append_tracks(&section_wavs, &song_out)?;

    Ok(())
}
