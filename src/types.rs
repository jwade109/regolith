use fraction::Fraction;
use reqwest::{Error as ReqError, StatusCode};

#[derive(Debug)]
pub enum CompileError
{
    Generic(String),
    GenericSyntax(String),
    Unexpected(String, Token, Literal),
    PreambleOrder(Literal, Literal, Literal),
    EmptyMeasure(Literal, Literal),
    InvalidSyntax(Literal),
    TimeSignatureViolation
    {
        measure: Measure,
        time_signature: Literal,
        nominal: TimeSignature,
    },
    FileError(std::io::Error),
    NetworkError(reqwest::Error),
    TrackTooLarge,
    TooManyRequests,
    DifferingMeasureCounts(u32, usize, u32, usize),
    EmptyTrack(u32),
}

impl From<std::io::Error> for CompileError
{
    fn from(error: std::io::Error) -> Self
    {
        CompileError::FileError(error)
    }
}

impl From<hound::Error> for CompileError
{
    fn from(error: hound::Error) -> Self
    {
        CompileError::Generic("Hound error".to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Literal
{
    pub literal: String,
    pub filename: String,
    pub lineno: usize,
    pub colno: usize,
    pub idno: usize
}

impl Literal
{
    pub fn to_string(&self) -> String
    {
        format!("\"{}\", line {}, column {}",
            self.literal, self.lineno, self.colno)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegoNote
{
    pub prefix: String,
    pub suffix: String,
    pub beats: Fraction
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DynamicLevel
{
    Pianissimo,
    Piano,
    Mezzopiano,
    Mezzoforte,
    Forte,
    Fortissimo
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ToneId(pub u8);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scale
{
    pub name: String,
    pub tone_id: ToneId,
    pub steps: Vec<u8>
}

impl Scale
{
    pub fn cmajor() -> Self
    {
        Scale
        {
            name: "cmajor".to_string(),
            tone_id: ToneId(13),
            steps: vec![2, 2, 1, 2, 2, 2, 1]
        }
    }
}

pub fn sample_scale(scale: &Scale, degree: u8) -> ToneId
{
    let octaves: u8 = (degree - 1) / scale.steps.len() as u8;
    let d: u8 = (degree - 1) % scale.steps.len() as u8;
    let ToneId(root) = scale.tone_id;
    let steps = scale.steps[0..(d as usize)].iter().sum::<u8>();
    ToneId((octaves * 12 as u8 + root + steps))
}

#[test]
fn test_sample_scale()
{
    let scale = Scale::cmajor();

    assert_eq!(sample_scale(&scale, 1), ToneId(13));
    assert_eq!(sample_scale(&scale, 2), ToneId(15));
    assert_eq!(sample_scale(&scale, 3), ToneId(17));
    assert_eq!(sample_scale(&scale, 4), ToneId(18));
    assert_eq!(sample_scale(&scale, 5), ToneId(20));
    assert_eq!(sample_scale(&scale, 6), ToneId(22));
    assert_eq!(sample_scale(&scale, 7), ToneId(24));
    assert_eq!(sample_scale(&scale, 8), ToneId(25));
}

pub type TimeSignature = (u8, u8);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token
{
    Track(u32),
    Tempo(u16),
    AbsolutePitch(ToneId),
    Note(RegoNote),
    Scale(Scale),
    ScaleDegree(u8),
    Dynamic(DynamicLevel),
    MeasureBar(bool, bool),
    Section(String),
    TimeSignature(TimeSignature),
    Endline(),
}

pub type CompileResult<T> = Result<T, CompileError>;

#[derive(Debug, Clone)]
pub struct NoteDecl
{
    pub note: RegoNote,
    pub note_literal: Literal,
    pub tone_id: ToneId
}

#[derive(Debug, Clone)]
pub struct Measure
{
    pub start: Literal,
    pub end: Literal,
    pub close: bool,
    pub open: bool,
    pub track: u32,
    pub notes: Vec<NoteDecl>
}

impl Measure
{
    pub fn count_beats(&self) -> Fraction
    {
        self.notes.iter().map(|n| n.note.beats).sum()
    }
}
