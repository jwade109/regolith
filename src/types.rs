use fraction::Fraction;

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
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RegoNote
{
    pub prefix: String,
    pub suffix: String,
    pub beats: Fraction
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DynamicLevel
{
    Pianissimo,
    Piano,
    Mezzopiano,
    Mezzoforte,
    Forte,
    Fortissimo
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Scale
{
    pub name: String,
    pub tone_id: u8,
    pub steps: Vec<u8>
}

impl Scale
{
    pub fn cmajor() -> Self
    {
        Scale
        {
            name: "cmajor".to_string(),
            tone_id: 13,
            steps: vec![2, 2, 1, 2, 2, 2, 1]
        }
    }
}

pub type TimeSignature = (u8, u8);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token
{
    Track(String),
    Tempo(u16),
    AbsolutePitch(u8),
    Note(RegoNote),
    StartRepeat(),
    EndRepeat(u8),
    Scale(Scale),
    ScaleDegree(i32),
    Dynamic(DynamicLevel),
    MeasureBar(),
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
    pub tone_id: u8
}

#[derive(Debug, Clone)]
pub struct Measure
{
    pub start: Literal,
    pub end: Literal,
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
