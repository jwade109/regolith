use crate::types::*;
use crate::lexer::{lex_markdown, lex_multiline_string};
use indoc::indoc;
use colored::Colorize;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum PreambleNode
{
    Tempo
    {
        token: & Token,
        tempo: u16,
    },
    Scale
    {
        token: & Token,
        scale: Scale,
    },
    DynamicLevel
    {
        token: & Token,
        level: DynamicLevel,
    },
    TimeSignature
    {
        token: & Token,
        ratio: TimeSignature,
    },
    Endline(& Token),
}

#[derive(Debug, Clone)]
pub enum StaffNode
{
    AbsolutePitch
    {
        token: & Token,
        pitch: ToneId,
    },
    Note
    {
        token: & Token,
        note: RegoNote,
    },
    Track
    {
        token: & Token,
        track_id: u32,
    },
    ScaleDegree
    {
        token: & Token,
        degree: u8,
    },
    MeasureBar
    {
        close: bool,
        open: bool,
        token: & Token,
    },
    Endline
    {
        token: & Token
    },
}

#[derive(Debug, Clone)]
pub struct SectionNode
{
    pub token: & Token,
    pub name: & str,
    pub preamble: Vec<PreambleNode>,
    pub measures: Vec<MeasureNode>
}

#[derive(Debug, Clone)]
pub struct MeasureNode
{
    pub start: Token,
    pub end: Token,
    pub staff: Vec<StaffNode>
}

pub type AST = Vec<SectionNode>;

struct Parser
{
    tokens: Vec<Token>,
    index: u32
}

impl Parser
{
    fn new(tokens: & Vec<Token>) -> Self
    {
        Parser { tokens, index: (tokens.len() - 1) as u32 }
    }

    fn peek(&self) -> Option<Token>
    {
        self.tokens.get(self.index as usize).map(|e| e.clone())
    }

    fn peek_ref(&self) -> Option<&Token>
    {
        self.tokens.get(self.index as usize)
    }

    fn peek_and_take_if<'b, F: Fn(&Token) -> bool>(&'b mut self, f: F) -> Option<Token>
    {
        let ret = self.peek_ref().map(f)?;
        if ret
        {
            self.take()
        }
        else
        {
            None
        }
    }

    fn take<'b>(&'b mut self) -> Option<&Token>
    {
        if self.index < 0
        {
            return None
        }
        self.index -= 1;
        self.tokens.get((self.index + 1) as usize)
    }
}

pub fn parse_to_ast(tokens: & Vec<Token>) -> CompileResult<AST>
{
    let mut parser = Parser::new(tokens);

    let mut sections = vec![];

    while let Some(_) = parser.peek()
    {
        let section: SectionNode = eat_section(&mut parser)?;
        sections.push(section);
    }

    Ok(sections)
}

fn eat_section(parser: & mut Parser) -> CompileResult<SectionNode>
{
    let should_take = |t: &Token|
    {
        match t.token
        {
            TokenValue::Section(_) => true,
            _ => false
        }
    };

    let (section_token) = parser.peek_and_take_if(should_take).ok_or(
        CompileError::GenericSyntax("Encountered EOF while parsing section block"))?;

    let section_name: &str = if let TokenValue::Section(ref name) = section_token.token
    {
        name
    }
    else
    {
        "<implicit-section>"
    };

    let mut first_staff: Option<&Token> = None;
    let mut preamble: Vec<PreambleNode> = vec![];
    let mut measures: Vec<MeasureNode> = vec![];

    // parse the preamble
    while let Some(token) = parser.peek_ref()
    {
        let node = match token.token
        {
            TokenValue::Dynamic(_) |
            TokenValue::TimeSignature(_) |
            TokenValue::Scale(_) |
            TokenValue::Tempo(_) |
            TokenValue::Endline() =>
            {
                eat_preamble_atomic(&mut parser)
            },
            TokenValue::Track(_) |
            TokenValue::MeasureBar(_, _) |
            TokenValue::AbsolutePitch(_) |
            TokenValue::ScaleDegree(_) |
            TokenValue::Note(_) |
            TokenValue::Section(_) => break,
        }?;

        preamble.push(node);
    }

    preamble = preamble.into_iter().filter(|node|
    {
        match node
        {
            &PreambleNode::Endline { .. } => false,
            _ => true,
        }
    }).collect();

    // parse the staff
    while let Some(token) = parser.peek()
    {
        let node: Option<MeasureNode> = match token.token
        {
            TokenValue::Section(_) => break,
            TokenValue::Dynamic(_) |
            TokenValue::Tempo(_) |
            TokenValue::Scale(_) |
            TokenValue::TimeSignature(_) =>
            {
                if let Some(ref first) = first_staff
                {
                    Err(CompileError::PreambleOrder(section_token, first, parser.take().unwrap()))
                }
                else
                {
                    Err(CompileError::GenericSyntax("Expected a staff element"))
                }
            },
            TokenValue::Endline() |
            TokenValue::MeasureBar(_, _) |
            TokenValue::Track(_) |
            TokenValue::ScaleDegree(_) |
            TokenValue::AbsolutePitch(_) |
            TokenValue::Note(_) =>
            {
                first_staff.get_or_insert(parser.take().unwrap());
                eat_measure_block(&mut parser)
            }
        }?;

        if let Some(n) = node
        {
            measures.push(n);
        }
    }

    Ok(SectionNode { token: section_token, name: section_name, preamble, measures })
}

fn atomic_token_to_staff_node(token: &Token) -> Option<StaffNode>
{
    match &token.token
    {
        TokenValue::Note(note) => Some(StaffNode::Note{ token, note: note.clone() }),
        TokenValue::Track(track_id) => Some(StaffNode::Track{ token, track_id: *track_id }),
        TokenValue::ScaleDegree(degree) => Some(StaffNode::ScaleDegree{ token, degree: *degree }),
        TokenValue::AbsolutePitch(pitch) => Some(StaffNode::AbsolutePitch{ token, pitch: *pitch }),
        TokenValue::MeasureBar(close, open) => Some(StaffNode::MeasureBar { token, close: *close, open: *open }),
        TokenValue::Endline() => Some(StaffNode::Endline{ token }),
        TokenValue::Tempo(_) |
        TokenValue::Dynamic(_) |
        TokenValue::Scale(_) |
        TokenValue::TimeSignature(_) |
        TokenValue::Section(_) => None
    }
}

fn atomic_token_to_preamble_node(token: & Token) -> Option<PreambleNode>
{
    match &token.token
    {
        TokenValue::Tempo(bpm) => Some(PreambleNode::Tempo{ token, tempo: *bpm }),
        TokenValue::Dynamic(level) => Some(PreambleNode::DynamicLevel{ token, level: *level }),
        TokenValue::Scale(scale) => Some(PreambleNode::Scale{ token, scale: scale.clone() }),
        TokenValue::TimeSignature(ratio) => Some(PreambleNode::TimeSignature{ token, ratio: *ratio }),
        TokenValue::Endline() => Some(PreambleNode::Endline(token)),
        TokenValue::Track(_) |
        TokenValue::ScaleDegree(_) |
        TokenValue::AbsolutePitch(_) |
        TokenValue::MeasureBar(_, _) |
        TokenValue::Section(_) |
        TokenValue::Note(_) => None
    }
}

fn eat_staff_atomic(parser: & mut Parser) -> CompileResult<StaffNode>
{
    if let Some(token) = parser.take()
    {
        if let Some(node) = atomic_token_to_staff_node(token)
        {
            return Ok(node);
        }
        else
        {
            return Err(CompileError::Unexpected("Expected an atomic staff token", token));
        }
    }

    Err(CompileError::GenericSyntax("Expected an atomic token, but nothing left"))
}

fn eat_preamble_atomic(parser: & mut Parser) -> CompileResult<PreambleNode>
{
    if let Some(token) = parser.take()
    {
        if let Some(node) = atomic_token_to_preamble_node(token)
        {
            return Ok(node);
        }
        else
        {
            return Err(CompileError::Unexpected("Expected an atomic preamble token", token));
        }
    }

    Err(CompileError::GenericSyntax("Expected a preamble token, but nothing left"))
}

fn eat_measure_block(parser: & mut Parser) -> CompileResult<Option<MeasureNode>>
{
    let mut staff = vec![];
    let mut skip_next_bar = true;

    let mut measure_start: Option<& Token> = None;
    let mut measure_end: Option<& Token> = None;

    while let Some(token) = parser.peek()
    {
        measure_start.get_or_insert(token);
        measure_end = Some(token);

        let node = match token.token
        {
            TokenValue::MeasureBar(_, _) =>
            {
                if skip_next_bar
                {
                    parser.take();
                    skip_next_bar = false;
                    None
                }
                else
                {
                    break
                }
            }
            TokenValue::Section(_) =>
            {
                break
            }
            TokenValue::AbsolutePitch(_) |
            TokenValue::ScaleDegree(_) |
            TokenValue::Endline() |
            TokenValue::Track(_) |
            TokenValue::Note(_) =>
            {
                skip_next_bar = false;
                Some(eat_staff_atomic(parser))
            },
            TokenValue::Dynamic(_) |
            TokenValue::Tempo(_) |
            TokenValue::TimeSignature(_) |
            TokenValue::Scale(_) => Some(Err(CompileError::Unexpected("Illegal token in measure block", token))),
        };

        if let Some(n) = node
        {
            staff.push(n?);
        }
    }

    let contains_endline = staff.iter().any(|node|
    {
        match node
        {
            StaffNode::Endline { .. } => true,
            _ => false,
        }
    });

    let start = measure_start.ok_or(
        CompileError::GenericSyntax("No start token for measure"))?;
    let end = measure_end.ok_or(
        CompileError::GenericSyntax("No end token for measure"))?;

    if staff.is_empty()
    {
        return Err(CompileError::EmptyMeasure(start, end))
    }

    if contains_endline && staff.len() == 1
    {
        return Ok(None)
    }

    staff = staff.into_iter().filter(|node|
    {
        match node
        {
            StaffNode::Endline { .. } => false,
            _ => true,
        }
    }).collect();

    Ok(Some(MeasureNode { start, end, staff }))
}

fn staff_node_to_string(node: &StaffNode, level: u32) -> String
{
    let pad = "                ";

    match node
    {
        StaffNode::AbsolutePitch{token, ..} => format!("{}[pitch] {}", pad, token.literal.literal),
        StaffNode::Note{token, ..} => format!("{}[note] {}", pad, token.literal.literal),
        StaffNode::Track{token, ..} => format!("{}[track] {}", pad, token.literal.literal),
        StaffNode::ScaleDegree{token, ..}  => format!("{}[relpitch] {}", pad, token.literal.literal),
        StaffNode::MeasureBar{token, ..}  => format!("{}[mb] {}", pad, token.literal.literal),
        StaffNode::Endline { .. } => format!("{}[endline]", pad),
    }
}

fn preamble_node_to_string(node: &PreambleNode) -> String
{
    let pad = "            ";

    match node
    {
        PreambleNode::Tempo{token, ..} => format!("{}[tempo] {}", pad, token.literal.literal),
        PreambleNode::DynamicLevel {token, .. } => format!("{}[dyn] {}", pad, token.literal.literal),
        PreambleNode::TimeSignature {token, .. } => format!("{}[time] {}", pad, token.literal.literal),
        PreambleNode::Scale {token, .. } => format!("{}[scale] {}", pad, token.literal.literal),
        PreambleNode::Endline(_) => format!("{}[endline]", pad),
    }
}

impl MeasureNode
{
    fn to_string(&self, level: u32) -> String
    {
        let mut segments = vec![];
        segments.push(format!("            [measure] {} .. {}", self.start.literal.literal, self.end.literal.literal));
        for n in &self.staff
        {
            segments.push(staff_node_to_string(n, level + 1))
        }
        segments.join("\n")
    }
}

fn section_to_string(section: &SectionNode) -> String
{
    let mut segments = vec![
        format!("    [section] {}", section.token.literal.literal)];

    segments.push("        [preamble]".to_string());
    for n in &section.preamble
    {
        segments.push(preamble_node_to_string(n));
    }

    segments.push("        [staff]".to_string());
    for m in &section.measures
    {
        segments.push(m.to_string(3));
    }

    segments.join("\n")
}

fn tree_to_string(tree: &AST) -> String
{
    let mut segments = vec!["[top]".to_string()];
    for section in tree
    {
        segments.push(section_to_string(section));
    }
    segments.push("[end]".to_string());
    segments.join("\n")
}

pub fn print_tree(tree: &AST)
{
    println!("{}", tree_to_string(tree))
}

fn pluralize<T: std::cmp::PartialEq<usize>>(count: T) -> &'static str
{
    if count == (1 as usize) { "" } else { "s" }
}

pub fn print_error(error: &CompileError)
{
    match error
    {
        CompileError::InvalidSyntax(literal) =>
        {
            println!("\n    {}\n\n    \"{}\", line {}, col {}\n",
                "Invalid syntax.".bold(),
                literal.literal, literal.lineno, literal.colno);
        },
        CompileError::Generic(msg) |
        CompileError::GenericSyntax(msg) =>
        {
            println!("\n  Generic error: {}\n", msg);
        },
        CompileError::Unexpected(msg, token) =>
        {
            println!("\n  Unexpected token: {}\n    Problematic token -- \"{}\", line {}, col {}\n",
                msg, token.literal.literal, token.literal.lineno, token.literal.colno);
        },
        CompileError::PreambleOrder(section, first, cur) =>
        {
            println!("\n  Cannot declare preamble element after staff has begun.");
            println!("    In this section --          \"{}\", line {}, col {}",
                section.literal.literal, section.literal.lineno, section.literal.colno);
            println!("    Staff begins here --        \"{}\", line {}, col {}",
                first.literal.literal, first.literal.lineno, first.literal.colno);
            println!("    Problematic element is --   \"{}\", line {}, col {}\n",
                cur.literal.literal, cur.literal.lineno, cur.literal.colno);
        },
        CompileError::EmptyMeasure(start, end) =>
        {
            println!("\n  Empty measure.");
            println!("    Measure starts here -- \"{}\", line {}, col {}",
                start.literal.literal, start.literal.lineno, start.literal.colno);
            println!("    Measure ends here -- \"{}\", line {}, col {}\n",
                end.literal.literal, end.literal.lineno, end.literal.colno);
        },
        CompileError::TimeSignatureViolation{ measure, time_signature, nominal } =>
        {
            println!("\n    {}\n", "Time signature violation.".bold());
            println!("    This measure is {} beats, which violates time signature {:?}",
                measure.count_beats(), nominal);
            println!("    Time signature declared here -- {}", time_signature.literal.to_string());
            println!("    Initiating element --  {}", measure.start.literal.to_string());
            println!("    Terminating element -- {}\n", measure.end.literal.to_string());

            // show_file(&measure.start, &measure.end, filename);
        },
        CompileError::NetworkError(e) =>
        {
            println!("\n    {}\n", "Network error.".bold());
            println!("    {:?}\n", e);
        },
        CompileError::FileError(e) =>
        {
            println!("\n    {}\n", "File IO error.".bold());
            println!("    {:?}\n", e);
        }
        CompileError::HoundError(e) =>
        {
            println!("\n    {}\n", "Hound error.".bold());
            println!("    {:?}\n", e);
        }
        CompileError::TrackTooLarge =>
        {
            println!("\n    {}\n\n", "Track too large; API call failed.".bold());
        }
        CompileError::DifferingMeasureCounts(ta, asize, tb, bsize) =>
        {
            println!("\n    {}\n", "Tracks have inconsistent length.".bold());
            println!("    Track {} has {} measure{};", ta, asize, pluralize(*asize));
            println!("    Track {} has {} measure{}.\n", tb, bsize, pluralize(*bsize));
        }
        CompileError::EmptyTrack(idx) =>
        {
            println!("\n    {}\n\n",
                format!("Track {} contains no measures.", idx).bold());
        }
    }
}

fn assert_ast_results(source: &str, ast_repr: &str)
{
    let tokens = lex_multiline_string(source).unwrap();
    let tree = parse_to_ast(&tokens).unwrap();
    let repr: String = tree_to_string(&tree);
    assert_eq!(repr, ast_repr);
}

#[test]
fn parsing_test()
{
    assert_ast_results("|: . ./2 :|",
        indoc! {"
        [top]
            [section] <implicit-section>
                [preamble]
                [staff]
                    [measure] |: .. :|
                        [note] .
                        [note] ./2
        [end]"});

    assert_ast_results("===BIG=== F3 . . F2 ./3 duh:3/2",
        indoc! {"
        [top]
            [section] ===BIG===
                [preamble]
                [staff]
                    [measure] F3 .. <eol>
                        [pitch] F3
                        [note] .
                        [note] .
                        [pitch] F2
                        [note] ./3
                        [note] duh:3/2
        [end]"});

    assert_ast_results("CMAJOR 4/4 |: -/2 -:3/2 -/2 -:3/2 .:5/2 |",
        indoc! {"
        [top]
            [section] <implicit-section>
                [preamble]
                    [scale] CMAJOR
                    [time] 4/4
                [staff]
                    [measure] |: .. |
                        [note] -/2
                        [note] -:3/2
                        [note] -/2
                        [note] -:3/2
                        [note] .:5/2
        [end]"});
}

fn test_parse_file(filename: &str)
{
    let path = Path::new(filename);
    let tokens = lex_markdown(path).unwrap();
    let tree = parse_to_ast(&tokens);
    assert!(tree.is_ok());
}

#[test]
fn parse_john_madden()
{
    test_parse_file("examples/hbjm.md");
}

#[test]
fn parse_mariah()
{
    test_parse_file("examples/mariah.md");
}

#[test]
fn parse_batman()
{
    test_parse_file("examples/batman.md");
}

#[test]
fn parse_the_sound_of_music()
{
    test_parse_file("examples/soundofmusic.md");
}
