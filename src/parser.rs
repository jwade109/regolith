use crate::lexer::{Literal, Token, lex_multiline_string, RegoNote, DynamicLevel, Scale};
use argparse::Parse;
use fraction::error::ParseError;
use indoc::indoc;
use fraction::Fraction;

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
        ratio: Fraction,
    },
}

#[derive(Debug, Clone)]
pub enum StaffNode
{
    RepeatBlock
    {
        start_literal: Literal,
        end_literal: Literal,
        count: u8,
        nodes: Vec<StaffNode>,
    },
    Section
    {
        literal: Literal,
        name: String,
        nodes: Vec<StaffNode>
    },
    AbsolutePitch
    {
        literal: Literal,
        pitch: u8,
    },
    Tempo // TODO delete
    {
        literal: Literal,
        tempo: u16,
    },
    Scale // TODO delete
    {
        literal: Literal,
        scale: Scale,
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
    BeatAssert
    {
        literal: Literal,
        beats: i32,
    },
    ScaleDegree
    {
        literal: Literal,
        degree: i32,
    },
    MeasureBar
    {
        literal: Literal,
    },
    DynamicLevel // TODO delete
    {
        literal: Literal,
        level: DynamicLevel,
    },
    TimeSignature // TODO delete
    {
        literal: Literal,
        ratio: Fraction,
    },
    Endline
    {
        literal: Literal
    },
}

#[derive(Debug)]
pub enum SyntaxError
{
    Generic(String),
    Unexpected(String, Token, Literal),
    PreambleOrder(Literal, Literal, Literal),
}

#[derive(Debug, Clone)]
pub struct SectionNode
{
    literal: Literal,
    name: String,
    preamble: Vec<StaffNode>, // TODO PreambleNode
    staff: Vec<StaffNode>
}

#[derive(Debug)]
pub struct AST
{
    pub sections: Vec<SectionNode>,
}

type ParseResult<T> = Result<T, SyntaxError>;

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

pub fn parse_to_ast(tokens: &Vec<(Literal, Token)>) -> ParseResult<AST>
{
    let mut parser = Parser::new(tokens);

    let mut sections = vec![];

    while let Some((literal, token)) = parser.peek()
    {
        let section = eat_section(&mut parser)?;
        sections.push(section);
    }

    Ok(AST{ sections })
}

fn eat_section(parser: &mut Parser) -> ParseResult<SectionNode>
{
    let (mut section_literal, section_token) = parser.peek_copy().ok_or(
        SyntaxError::Generic("Encountered EOF while parsing section block".to_string()))?;
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

    let mut first_staff: Option<(Literal, Token)> = None;

    let mut staff = vec![];
    let mut preamble = vec![];

    while let Some((literal, token)) = parser.peek_copy()
    {
        if let Token::Section(_) = token
        {
            // it's another section header; we're done
            break
        }

        let node = match token
        {
            Token::Endline() => eat_atomic(parser),
            Token::Dynamic(_) |
            Token::Tempo(_) |
            Token::Scale(_) |
            Token::TimeSignature(_) =>
            {
                if let Some(first) = first_staff.clone()
                {
                    Err(SyntaxError::PreambleOrder(section_literal.clone(), first.0, literal))
                }
                else
                {
                    eat_atomic(parser)
                }
            },
            Token::MeasureBar() |
            Token::Track(_) |
            Token::ScaleDegree(_) |
            Token::BeatAssert(_) |
            Token::AbsolutePitch(_) |
            Token::Note(_) =>
            {
                first_staff.get_or_insert((literal, token));
                eat_atomic(parser)
            },
            Token::StartRepeat() =>
            {
                first_staff.get_or_insert((literal, token));
                eat_repeat_block(parser)
            },
            _ => Err(SyntaxError::Unexpected("Illegal token in section".to_string(), token, literal)),
        }?;

        if first_staff.is_none()
        {
            preamble.push(node);
        }
        else
        {
            staff.push(node);
        }
    }

    Ok(SectionNode { literal: section_literal, name: section_name, preamble, staff })
}

fn eat_repeat_block(parser: &mut Parser) -> ParseResult<StaffNode>
{
    let (start_literal, start_token) = parser.take()
        .ok_or(SyntaxError::Generic("Expected token at start of repeat block".to_string()))?;
    if let Token::StartRepeat() = start_token
    {
        // great!
    }
    else
    {
        return Err(SyntaxError::Unexpected(
            "Expected a repeat block initiator".to_string(), start_token, start_literal));
    }
    let nodes = eat_repeat_block_interior(parser)?;
    let (end_literal, end_token) = parser.take()
        .ok_or(SyntaxError::Generic("Unterminated repeat block".to_string()))?;
    if let Token::EndRepeat(count) = end_token
    {
        return Ok(StaffNode::RepeatBlock{ start_literal, end_literal, count, nodes });
    }
    Err(SyntaxError::Unexpected("Expected end repeat block token".to_string(),
        end_token.clone(), end_literal.clone()))
}

fn atomic_token_to_ast_node(token: Token, literal: Literal) -> Option<StaffNode>
{
    match token
    {
        Token::Note(note) => Some(StaffNode::Note{ literal, note }),
        Token::Track(track_id) => Some(StaffNode::Track{ literal, track_id }),
        Token::Tempo(bpm) => Some(StaffNode::Tempo{ literal, tempo: bpm }),
        Token::Dynamic(level) => Some(StaffNode::DynamicLevel{ literal, level }),
        Token::Scale(scale) => Some(StaffNode::Scale{ literal, scale: scale.clone() }),
        Token::ScaleDegree(degree) => Some(StaffNode::ScaleDegree{ literal, degree }),
        Token::AbsolutePitch(pitch) => Some(StaffNode::AbsolutePitch{ literal, pitch }),
        Token::BeatAssert(beats) => Some(StaffNode::BeatAssert { literal, beats }),
        Token::MeasureBar() => Some(StaffNode::MeasureBar { literal }),
        Token::TimeSignature(ratio) => Some(StaffNode::TimeSignature{ literal, ratio }),
        Token::Endline() => Some(StaffNode::Endline{ literal }),
        Token::Section(_) |
        Token::EndRepeat(_) |
        Token::StartRepeat() => None,
    }
}

fn eat_atomic(parser: &mut Parser) -> ParseResult<StaffNode>
{
    if let Some((literal, token)) = parser.take()
    {
        if let Some(node) = atomic_token_to_ast_node(token.clone(), literal.clone())
        {
            return Ok(node);
        }
        else
        {
            return Err(SyntaxError::Unexpected(
                "Expected an atomic token, but got".to_string(), token, literal));
        }
    }

    Err(SyntaxError::Generic("Expected an atomic token, but nothing left".to_string()))
}

fn eat_repeat_block_interior(parser: &mut Parser) -> ParseResult<Vec<StaffNode>>
{
    let mut nodes = vec![];

    while let Some((literal, token)) = parser.peek()
    {
        if let Token::EndRepeat(_) = token
        {
            break;
        }

        let node = match token
        {
            Token::Note(_) |
            Token::AbsolutePitch(_) |
            Token::Endline() |
            Token::MeasureBar() => eat_atomic(parser),
            _ => Err(SyntaxError::Unexpected("Illegal token in repeat block".to_string(),
                token.clone(), literal.clone())),
        }?;

        nodes.push(node);
    }

    Ok(nodes)
}

fn node_to_string(node: &StaffNode, level: u32) -> String
{
    let pad = (0..level*3).map(|_| " ").collect::<String>();

    match node
    {
        StaffNode::Tempo{literal, ..} => format!("{}[tempo] {}", pad, literal.literal),
        StaffNode::AbsolutePitch{literal, ..} => format!("{}[pitch] {}", pad, literal.literal),
        StaffNode::Scale{literal, ..} => format!("{}[scale] {}", pad, literal.literal),
        StaffNode::Note{literal, ..} => format!("{}[note] {}", pad, literal.literal),
        StaffNode::Track{literal, ..} => format!("{}[track] {}", pad, literal.literal),
        StaffNode::BeatAssert{literal, ..} => format!("{}[beats] {}", pad, literal.literal),
        StaffNode::ScaleDegree{literal, ..}  => format!("{}[relpitch] {}", pad, literal.literal),
        StaffNode::MeasureBar{literal, ..}  => format!("{}[mb] {}", pad, literal.literal),
        StaffNode::DynamicLevel{literal, ..}  => format!("{}[dyn] {}", pad, literal.literal),
        StaffNode::TimeSignature { literal, .. } => format!("{}[ts] {}", pad, literal.literal),
        StaffNode::Endline { .. } => format!("{}[endline]", pad),
        StaffNode::RepeatBlock{start_literal, end_literal, count, nodes} =>
        {
            let mut segments = vec![
                format!("{}[repeat] x{}", pad, count)];

            for n in nodes
            {
                segments.push(node_to_string(n, level + 1));
            }

            segments.join("\n")
        },
        StaffNode::Section{literal, name, nodes} =>
        {
            let mut segments = vec![
                format!("{}[section] {}", pad, literal.literal)];

            for n in nodes
            {
                segments.push(node_to_string(n, level + 1));
            }

            segments.join("\n")
        },
    }
}

fn section_to_string(section: &SectionNode) -> String
{
    let mut segments = vec![
        format!("   [section] {}", section.literal.literal)];

    segments.push("      [preamble]".to_string());
    for n in &section.preamble
    {
        segments.push(node_to_string(n, 3));
    }

    segments.push("      [staff]".to_string());
    for n in &section.staff
    {
        segments.push(node_to_string(n, 3));
    }

    segments.join("\n")
}

fn tree_to_string(tree: &AST) -> String
{
    let mut segments = vec!["[top]".to_string()];
    for section in &tree.sections
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

pub fn print_parse_error(error: &SyntaxError)
{
    match error
    {
        SyntaxError::Generic(msg) =>
        {
            println!("\n  Generic parse error: {}\n", msg);
        },
        SyntaxError::Unexpected(msg, token, literal) =>
        {
            println!("\n  Unexpected token: {} - {:?}, \"{}\", line {}, col {}\n",
                msg, token, literal.literal, literal.lineno, literal.colno);
        },
        SyntaxError::PreambleOrder(section, first, cur) =>
        {
            println!("\n  Cannot declare preamble element after staff has begun.");
            println!("    In this section --          \"{}\", line {}, col {}",
                section.literal, section.lineno, section.colno);
            println!("    Staff begins here --        \"{}\", line {}, col {}",
                first.literal, first.lineno, first.colno);
            println!("    Problematic element is --   \"{}\", line {}, col {}\n",
                cur.literal, cur.lineno, cur.colno);
        }
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
    assert_ast_results("[: . ./2 :]",
        indoc! {"
        [top]
           [section] <implicit-section>
              [preamble]
              [staff]
                 [repeat] x1
                    [note] .
                    [note] ./2
                 [endline]
        [end]"});

    assert_ast_results("---BIG--- F3 . . F2 ./3 duh:3/2",
        indoc! {"
        [top]
           [section] ---BIG---
              [preamble]
              [staff]
                 [pitch] F3
                 [note] .
                 [note] .
                 [pitch] F2
                 [note] ./3
                 [note] duh:3/2
                 [endline]
        [end]"});
}
