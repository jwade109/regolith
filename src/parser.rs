use crate::types::*;
use crate::lexer::{lex_markdown, lex_multiline_string};
use indoc::indoc;
use colored::Colorize;


struct Parser
{
    tokens: Vec<(Literal, Token)>
}

#[derive(Debug, Clone)]
pub enum PreambleNode
{
    Tempo
    {
        literal: Literal,
        tempo: u16,
    },
    Scale
    {
        literal: Literal,
        scale: Scale,
    },
    DynamicLevel
    {
        literal: Literal,
        level: DynamicLevel,
    },
    TimeSignature
    {
        literal: Literal,
        ratio: TimeSignature,
    },
    Endline(Literal),
}

#[derive(Debug, Clone)]
pub enum StaffNode
{
    AbsolutePitch
    {
        literal: Literal,
        pitch: u8,
    },
    Note
    {
        literal: Literal,
        note: RegoNote,
    },
    Track
    {
        literal: Literal,
        track_id: String,
    },
    ScaleDegree
    {
        literal: Literal,
        degree: i32,
    },
    MeasureBar
    {
        close: bool,
        open: bool,
        literal: Literal,
    },
    Endline
    {
        literal: Literal
    },
}

#[derive(Debug, Clone)]
pub struct SectionNode
{
    pub literal: Literal,
    pub name: String,
    pub preamble: Vec<PreambleNode>,
    pub measures: Vec<MeasureNode>
}

#[derive(Debug, Clone)]
pub struct MeasureNode
{
    pub start: (Literal, Token),
    pub end: (Literal, Token),
    pub staff: Vec<StaffNode>
}

pub type AST = Vec<SectionNode>;

impl Parser
{
    fn new(tokens: &Vec<(Literal, Token)>) -> Self
    {
        Parser { tokens: tokens.iter().rev().map(
            |(l, t)| (l.clone(), t.clone())).collect() }
    }

    fn peek(&self) -> Option<&(Literal, Token)>
    {
        return self.tokens.last()
    }

    fn peek_copy(&self) -> Option<(Literal, Token)>
    {
        return self.tokens.last().cloned()
    }

    fn take(&mut self) -> Option<(Literal, Token)>
    {
        return self.tokens.pop()
    }
}

pub fn parse_to_ast(tokens: &Vec<(Literal, Token)>) -> CompileResult<AST>
{
    let mut parser = Parser::new(tokens);

    let mut sections = vec![];

    while let Some((_, _)) = parser.peek()
    {
        let section = eat_section(&mut parser)?;
        sections.push(section);
    }

    Ok(sections)
}

fn eat_section(parser: &mut Parser) -> CompileResult<SectionNode>
{
    let (mut section_literal, section_token) = parser.peek_copy().ok_or(
        CompileError::GenericSyntax("Encountered EOF while parsing section block".to_string()))?;
    let section_name = if let Token::Section(ref name) = section_token
    {
        parser.take();
        name.clone()
    }
    else
    {
        section_literal.literal = "<implicit-section>".to_string();
        "".to_string()
    };

    let mut first_staff: Option<Literal> = None;
    let mut preamble: Vec<PreambleNode> = vec![];
    let mut measures: Vec<MeasureNode> = vec![];

    // parse the preamble
    while let Some((_, token)) = parser.peek_copy()
    {
        let node = match token
        {
            Token::Dynamic(_) |
            Token::TimeSignature(_) |
            Token::Scale(_) |
            Token::Tempo(_) |
            Token::Endline() =>
            {
                eat_preamble_atomic(parser)
            },
            Token::Track(_) |
            Token::MeasureBar(_, _) |
            Token::AbsolutePitch(_) |
            Token::ScaleDegree(_) |
            Token::Note(_) |
            Token::Section(_) => break,
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
    while let Some((literal, token)) = parser.peek_copy()
    {
        let node = match token
        {
            Token::Section(_) => break,
            Token::Dynamic(_) |
            Token::Tempo(_) |
            Token::Scale(_) |
            Token::TimeSignature(_) =>
            {
                if let Some(ref first) = first_staff
                {
                    Err(CompileError::PreambleOrder(section_literal.clone(), first.clone(), literal))
                }
                else
                {
                    Err(CompileError::GenericSyntax("Expected a staff element".to_string()))
                }
            },
            Token::Endline() |
            Token::MeasureBar(_, _) |
            Token::Track(_) |
            Token::ScaleDegree(_) |
            Token::AbsolutePitch(_) |
            Token::Note(_) =>
            {
                first_staff.get_or_insert(literal);
                eat_measure_block(parser)
            }
        }?;

        if let Some(n) = node
        {
            measures.push(n);
        }
    }

    Ok(SectionNode { literal: section_literal, name: section_name, preamble, measures })
}

fn atomic_token_to_staff_node(token: Token, literal: Literal) -> Option<StaffNode>
{
    match token
    {
        Token::Note(note) => Some(StaffNode::Note{ literal, note }),
        Token::Track(track_id) => Some(StaffNode::Track{ literal, track_id }),
        Token::ScaleDegree(degree) => Some(StaffNode::ScaleDegree{ literal, degree }),
        Token::AbsolutePitch(pitch) => Some(StaffNode::AbsolutePitch{ literal, pitch }),
        Token::MeasureBar(close, open) => Some(StaffNode::MeasureBar { literal, close, open }),
        Token::Endline() => Some(StaffNode::Endline{ literal }),
        Token::Tempo(_) |
        Token::Dynamic(_) |
        Token::Scale(_) |
        Token::TimeSignature(_) |
        Token::Section(_) => None
    }
}

fn atomic_token_to_preamble_node(token: Token, literal: Literal) -> Option<PreambleNode>
{
    match token
    {
        Token::Tempo(bpm) => Some(PreambleNode::Tempo{ literal, tempo: bpm }),
        Token::Dynamic(level) => Some(PreambleNode::DynamicLevel{ literal, level }),
        Token::Scale(scale) => Some(PreambleNode::Scale{ literal, scale: scale.clone() }),
        Token::TimeSignature(ratio) => Some(PreambleNode::TimeSignature{ literal, ratio }),
        Token::Endline() => Some(PreambleNode::Endline(literal)),
        Token::Track(_) |
        Token::ScaleDegree(_) |
        Token::AbsolutePitch(_) |
        Token::MeasureBar(_, _) |
        Token::Section(_) |
        Token::Note(_) => None
    }
}

fn eat_staff_atomic(parser: &mut Parser) -> CompileResult<StaffNode>
{
    if let Some((literal, token)) = parser.take()
    {
        if let Some(node) = atomic_token_to_staff_node(token.clone(), literal.clone())
        {
            return Ok(node);
        }
        else
        {
            return Err(CompileError::Unexpected(
                "Expected an atomic staff token".to_string(), token, literal));
        }
    }

    Err(CompileError::GenericSyntax("Expected an atomic token, but nothing left".to_string()))
}

fn eat_preamble_atomic(parser: &mut Parser) -> CompileResult<PreambleNode>
{
    if let Some((literal, token)) = parser.take()
    {
        if let Some(node) = atomic_token_to_preamble_node(token.clone(), literal.clone())
        {
            return Ok(node);
        }
        else
        {
            return Err(CompileError::Unexpected(
                "Expected an atomic preamble token".to_string(), token, literal));
        }
    }

    Err(CompileError::GenericSyntax("Expected a preamble token, but nothing left".to_string()))
}

fn eat_measure_block(parser: &mut Parser) -> CompileResult<Option<MeasureNode>>
{
    let mut staff = vec![];
    let mut skip_next_bar = true;

    let mut measure_start: Option<(Literal, Token)> = None;
    let mut measure_end: Option<(Literal, Token)> = None;

    while let Some((literal, token)) = parser.peek()
    {
        measure_start.get_or_insert((literal.clone(), token.clone()));
        measure_end = Some((literal.clone(), token.clone()));

        let node = match token
        {
            Token::MeasureBar(_, _) =>
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
            Token::Section(_) =>
            {
                break
            }
            Token::AbsolutePitch(_) |
            Token::ScaleDegree(_) |
            Token::Endline() |
            Token::Track(_) |
            Token::Note(_) =>
            {
                skip_next_bar = false;
                Some(eat_staff_atomic(parser))
            },
            Token::Dynamic(_) |
            Token::Tempo(_) |
            Token::TimeSignature(_) |
            Token::Scale(_) => Some(Err(CompileError::Unexpected(
                "Illegal token in measure block".to_string(),
                token.clone(), literal.clone()))),
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
        CompileError::GenericSyntax("No start token for measure".to_string()))?;
    let end = measure_end.ok_or(
        CompileError::GenericSyntax("No end token for measure".to_string()))?;

    if staff.is_empty()
    {
        return Err(CompileError::EmptyMeasure(start.0, end.0))
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
        StaffNode::AbsolutePitch{literal, ..} => format!("{}[pitch] {}", pad, literal.literal),
        StaffNode::Note{literal, ..} => format!("{}[note] {}", pad, literal.literal),
        StaffNode::Track{literal, ..} => format!("{}[track] {}", pad, literal.literal),
        StaffNode::ScaleDegree{literal, ..}  => format!("{}[relpitch] {}", pad, literal.literal),
        StaffNode::MeasureBar{literal, ..}  => format!("{}[mb] {}", pad, literal.literal),
        StaffNode::Endline { .. } => format!("{}[endline]", pad),
    }
}

fn preamble_node_to_string(node: &PreambleNode, level: u32) -> String
{
    let pad = "            ";

    match node
    {
        PreambleNode::Tempo{literal, ..} => format!("{}[tempo] {}", pad, literal.literal),
        PreambleNode::DynamicLevel { literal, .. } => format!("{}[dyn] {}", pad, literal.literal),
        PreambleNode::TimeSignature { literal, .. } => format!("{}[time] {}", pad, literal.literal),
        PreambleNode::Scale { literal, .. } => format!("{}[scale] {}", pad, literal.literal),
        PreambleNode::Endline(literal) => format!("{}[endline]", pad),
    }
}

impl MeasureNode
{
    fn to_string(&self, level: u32) -> String
    {
        let mut segments = vec![];
        segments.push(format!("            [measure] {} .. {}", self.start.0.literal, self.end.0.literal));
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
        format!("    [section] {}", section.literal.literal)];

    segments.push("        [preamble]".to_string());
    for n in &section.preamble
    {
        segments.push(preamble_node_to_string(n, 3));
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

pub fn print_error(error: &CompileError)
{
    match error
    {
        CompileError::InvalidSyntax(literal) =>
        {
            println!("\n  Invalid syntax: \"{}\", line {}, col {}\n",
                literal.literal, literal.lineno, literal.colno);
        },
        CompileError::Generic(msg) |
        CompileError::GenericSyntax(msg) =>
        {
            println!("\n  Generic parse error: {}\n", msg);
        },
        CompileError::Unexpected(msg, token, literal) =>
        {
            println!("\n  Unexpected token: {}\n    Problematic token -- \"{}\", line {}, col {}\n",
                msg, literal.literal, literal.lineno, literal.colno);
        },
        CompileError::PreambleOrder(section, first, cur) =>
        {
            println!("\n  Cannot declare preamble element after staff has begun.");
            println!("    In this section --          \"{}\", line {}, col {}",
                section.literal, section.lineno, section.colno);
            println!("    Staff begins here --        \"{}\", line {}, col {}",
                first.literal, first.lineno, first.colno);
            println!("    Problematic element is --   \"{}\", line {}, col {}\n",
                cur.literal, cur.lineno, cur.colno);
        },
        CompileError::EmptyMeasure(start, end) =>
        {
            println!("\n  Empty measure.");
            println!("    Measure starts here -- \"{}\", line {}, col {}",
                start.literal, start.lineno, start.colno);
            println!("    Measure ends here -- \"{}\", line {}, col {}\n",
                end.literal, end.lineno, end.colno);
        },
        CompileError::TimeSignatureViolation{ measure, time_signature, nominal } =>
        {
            println!("\n    {}\n", "Time signature violation.".underline());
            println!("    This measure is {} beats, which violates time signature {:?}",
                measure.count_beats(), nominal);
            println!("    Time signature declared here -- {}", time_signature.to_string());
            println!("    Initiating element --  {}", measure.start.to_string());
            println!("    Terminating element -- {}\n", measure.end.to_string());

            // show_file(&measure.start, &measure.end, filename);
        },
    }
}

fn assert_ast_results(source: &str, ast_repr: &str)
{
    let tokens = lex_multiline_string(source).unwrap();
    let tree = parse_to_ast(&tokens).unwrap();
    let repr = tree_to_string(&tree);
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
    let tokens = lex_markdown(filename).unwrap();
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
