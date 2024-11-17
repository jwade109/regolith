use crate::types::*;
use crate::parser::*;
use fraction::Fraction;
use std::collections::HashMap;

type TrackMap<> = HashMap<u32, Vec<Measure<>>>;

#[derive(Debug)]
pub struct Section<>
{
    pub id: u32,
    pub name: & str,
    pub tempo: u16,
    pub dynamic: DynamicLevel,
    pub scale: Scale,
    pub time_signature: Option<(& Token, TimeSignature)>,
    pub tracks: TrackMap<>
}

impl<> Section<>
{
    pub fn to_string(&self) -> String
    {
        let mut sections = vec![
            format!("[section] \"{}\", {}, {} bpm, {:?}, {}, {:?}",
            self.id, self.name, self.tempo, self.dynamic, self.scale.name,
            self.time_signature)];
        for (_, measures) in &self.tracks
        {
            for measure in measures
            {
                let s = format!("  [measure] [track \"{}\"] ({} beats) {}",
                    measure.track, measure.count_beats(),
                    measure.notes.iter().map(|n| n.decl.literal.literal.clone())
                    .collect::<Vec<_>>().join(" "));
                sections.push(s);
            }
        }
        sections.join("\n")
    }
}

#[derive(Debug)]
pub struct Composition<>
{
    pub sections: Vec<Section<>>
}

struct CompositionState<>
{
    tempo: u16,
    dynamic: DynamicLevel,
    scale: Scale,
    time_signature: Option<(& Token, TimeSignature)>,
    tone_id: ToneId,
    track: u32
}

impl<> CompositionState<>
{
    fn defaults() -> Self
    {
        CompositionState
        {
            tempo: 120,
            dynamic: DynamicLevel::Mezzoforte,
            scale: Scale::cmajor(),
            time_signature: None,
            tone_id: ToneId(13), // TODO
            track: 0
        }
    }
}

fn assert_consistent_measure_counts<>(section: &Section) -> CompileResult<()>
{
    let mut baseline = None;
    for (track_id, measures) in &section.tracks
    {
        if measures.is_empty()
        {
            return Err(CompileError::EmptyTrack(*track_id))
        }

        if let Some((btid, count)) = baseline
        {
            if measures.len() != count
            {
                return Err(CompileError::DifferingMeasureCounts(btid, count, *track_id, measures.len()));
            }
        }
        else
        {
            baseline = Some((*track_id, measures.len()));
        }
    }

    Ok(())
}

fn make_section<>(id: u32, section: & SectionNode, state: &mut CompositionState) -> CompileResult<Section<>>
{
    for node in &section.preamble
    {
        match node
        {
            PreambleNode::DynamicLevel { token: _, level } =>
            {
                state.dynamic = level.clone();
            },
            PreambleNode::Scale { token: _, scale } =>
            {
                state.scale = scale.clone();
            },
            PreambleNode::TimeSignature { token, ratio } =>
            {
                state.time_signature = Some((token, ratio.clone()));
            },
            PreambleNode::Tempo { token: _, tempo } =>
            {
                state.tempo = *tempo;
            }
            PreambleNode::Endline(_) => (),
        }
    }

    let mut tracks: TrackMap = TrackMap::new();

    for meas in &section.measures
    {
        let mut notes: Vec<NoteDecl> = vec![];

        for snode in &meas.staff
        {
            match snode
            {
                StaffNode::Note { token, note } =>
                {
                    let n = NoteDecl
                    {
                        note,
                        decl: token,
                        tone_id: state.tone_id
                    };
                    notes.push(n);
                },
                StaffNode::AbsolutePitch { token: _, pitch } =>
                {
                    state.tone_id = *pitch;
                },
                StaffNode::ScaleDegree { token: _, degree } =>
                {
                    state.tone_id = sample_scale(&state.scale, *degree);
                },
                StaffNode::Track { token: _, track_id } =>
                {
                    state.track = *track_id;
                },
                StaffNode::MeasureBar { token, .. } |
                StaffNode::Endline { token } =>
                {
                    return Err(CompileError::InvalidSyntax(token.literal.clone()));
                },
            }
        }

        if notes.is_empty()
        {
            continue;
        }

        let open = if let TokenValue::MeasureBar(_, open) = meas.start.token
        {
            open
        }
        else
        {
            false
        };
        let close = if let TokenValue::MeasureBar(close, _) = meas.end.token
        {
            close
        }
        else
        {
            false
        };

        let m = Measure
        {
            start: meas.start,
            end: meas.end,
            close,
            open,
            track: state.track.clone(),
            notes
        };

        if tracks.get(&m.track).is_none()
        {
            tracks.insert(m.track, vec![]);
        }

        tracks.get_mut(&m.track).unwrap().push(m);
    }

    if let Some(ts) = &state.time_signature
    {
        for (track_id, measures) in &tracks
        {
            for meas in measures
            {
                let beats = meas.count_beats();
                if beats == Fraction::new(0u64, 1u64)
                {
                    continue;
                }

                let dirty = Fraction::new(ts.1.0, 1u64);
                if beats != dirty
                {
                    return Err(CompileError::TimeSignatureViolation
                    {
                        measure: meas.clone(),
                        time_signature: ts.0,
                        nominal: ts.1
                    });
                }
            }
        }
    }

    let s = Section
    {
        id,
        name: section.name.clone(),
        tempo: state.tempo.clone(),
        dynamic: state.dynamic.clone(),
        scale: state.scale.clone(),
        time_signature: state.time_signature.clone(),
        tracks
    };

    assert_consistent_measure_counts(&s)?;

    return Ok(s)
}

pub fn do_semantics<>(tree: & AST) -> CompileResult<Composition<>>
{
    let mut state = CompositionState::defaults();

    let mut sections = vec![];
    for (id, node) in tree.iter().enumerate()
    {
        let s = make_section(id as u32, node, &mut state)?;
        sections.push(s);
    }

    Ok(Composition{ sections })
}
