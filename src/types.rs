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
}

impl From<std::io::Error> for CompileError
{
    fn from(error: std::io::Error) -> Self
    {
        CompileError::FileError(error)
    }
}

impl From<reqwest::Error> for CompileError
{
    fn from(error: reqwest::Error) -> Self
    {
        if let Some(status) = error.status()
        {
            match status
            {
                StatusCode::PAYLOAD_TOO_LARGE => CompileError::TrackTooLarge,
                _ => CompileError::NetworkError(error)
            }
        }
        else
        {
            CompileError::NetworkError(error)
        }

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

#[derive(Debug, Clone)]
pub struct RegoNote
{
    pub prefix: String,
    pub suffix: String,
    pub beats: Fraction
}

#[derive(Debug, Clone)]
pub enum DynamicLevel
{
    Pianissimo,
    Piano,
    Mezzopiano,
    Mezzoforte,
    Forte,
    Fortissimo
}

#[derive(Debug, Copy, Clone)]
pub struct ToneId(pub u8);

#[derive(Debug, Clone)]
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

pub fn sample_scale(scale: &Scale, degree: usize) -> ToneId
{
    // TODO this might panic if degree is too big or negative!
    let ToneId(root) = scale.tone_id;
    ToneId(scale.steps[0..degree].iter().sum::<u8>())
}

pub type TimeSignature = (u8, u8);

#[derive(Debug, Clone)]
pub enum Token
{
    Track(String),
    Tempo(u16),
    AbsolutePitch(ToneId),
    Note(RegoNote),
    Scale(Scale),
    ScaleDegree(i32),
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
    pub track: String,
    pub notes: Vec<NoteDecl>
}

impl Measure
{
    pub fn count_beats(&self) -> Fraction
    {
        self.notes.iter().map(|n| n.note.beats).sum()
    }
}
