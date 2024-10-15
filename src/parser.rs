use crate::lexer::{Literal, Token, lex_multiline_string, RegoNote, DynamicLevel, Scale};
use indoc::indoc;

struct Parser
{
    tokens: Vec<(Literal, Token)>
}

#[derive(Debug, Clone)]
pub enum ASTNode
{
    RepeatBlock
    {
        start_literal: Literal,
        end_literal: Literal,
        count: u8,
        nodes: Vec<ASTNode>,
    },
    Section
    {
        literal: Literal,
        name: String,
        nodes: Vec<ASTNode>
    },
    AbsolutePitch
    {
        literal: Literal,
        pitch: u8,
    },
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
    DynamicLevel
    {
        literal: Literal,
        level: DynamicLevel,
    },
}

#[derive(Debug)]
pub enum SyntaxError
{
    Generic(String),
    Unexpected(String, Token, Literal),
}

#[derive(Debug)]
pub struct AST
{
    pub nodes: Vec<ASTNode>,
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

    let mut nodes = vec![];

    while let Some((literal, token)) = parser.peek()
    {
        let node = match token
        {
            Token::AbsolutePitch(_) => eat_atomic(&mut parser),
            Token::Tempo(_) => eat_atomic(&mut parser),
            Token::Scale(_) => eat_atomic(&mut parser),
            Token::Track(_) => eat_atomic(&mut parser),
            Token::Note(_) => eat_atomic(&mut parser),
            Token::BeatAssert(_) => eat_atomic(&mut parser),
            Token::ScaleDegree(_) => eat_atomic(&mut parser),
            Token::MeasureBar() => eat_atomic(&mut parser),
            Token::Dynamic(_) => eat_atomic(&mut parser),
            Token::Section(_) => eat_section(&mut parser),
            Token::StartRepeat() => eat_repeat_block(&mut parser),
            Token::EndRepeat(_) => Err(SyntaxError::Unexpected(
                "Unexpected repeat block terminator".to_string(),
                token.clone(), literal.clone())),
        }?;

        nodes.push(node);
    }

    Ok(AST{ nodes })
}

fn eat_section(parser: &mut Parser) -> ParseResult<ASTNode>
{
    let (section_literal, section_token) = parser.take().ok_or(
        SyntaxError::Generic("Expected a section header".to_string()))?;
    let section_name = if let Token::Section(name) = section_token { name }
        else { return Err(SyntaxError::Unexpected(
            "Expected a section header".to_string(), section_token, section_literal)); };

    let mut nodes = vec![];

    while let Some((literal, token)) = parser.peek_copy()
    {
        if let Token::Section(_) = token
        {
            // it's another section header; we're done
            break
        }

        let node = match token
        {
            Token::Note(_) |
            Token::Dynamic(_) |
            Token::Tempo(_) |
            Token::MeasureBar() |
            Token::AbsolutePitch(_) |
            Token::Scale(_) |
            Token::Track(_) => eat_atomic(parser),
            Token::StartRepeat() => eat_repeat_block(parser),
            _ => Err(SyntaxError::Unexpected("Illegal token in section".to_string(), token, literal)),
        }?;

        nodes.push(node);
    }

    Ok(ASTNode::Section { literal: section_literal, name: section_name, nodes })
}

fn eat_repeat_block(parser: &mut Parser) -> ParseResult<ASTNode>
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
        return Ok(ASTNode::RepeatBlock{ start_literal, end_literal, count, nodes });
    }
    Err(SyntaxError::Unexpected("Expected end repeat block token".to_string(),
        end_token.clone(), end_literal.clone()))
}

fn atomic_token_to_ast_node(token: Token, literal: Literal) -> Option<ASTNode>
{
    match token
    {
        Token::Note(note) => Some(ASTNode::Note{ literal, note }),
        Token::Track(track_id) => Some(ASTNode::Track{ literal, track_id }),
        Token::Tempo(bpm) => Some(ASTNode::Tempo{ literal, tempo: bpm }),
        Token::Dynamic(level) => Some(ASTNode::DynamicLevel{ literal, level }),
        Token::Scale(scale) => Some(ASTNode::Scale{ literal, scale: scale.clone() }),
        Token::ScaleDegree(degree) => Some(ASTNode::ScaleDegree{ literal, degree }),
        Token::AbsolutePitch(pitch) => Some(ASTNode::AbsolutePitch{ literal, pitch }),
        Token::BeatAssert(beats) => Some(ASTNode::BeatAssert { literal, beats }),
        Token::MeasureBar() => Some(ASTNode::MeasureBar { literal }),
        Token::Section(_) |
        Token::EndRepeat(_) |
        Token::StartRepeat() => None,
    }
}

fn eat_atomic(parser: &mut Parser) -> ParseResult<ASTNode>
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

fn eat_repeat_block_interior(parser: &mut Parser) -> ParseResult<Vec<ASTNode>>
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
            Token::Note(_) => eat_atomic(parser),
            Token::AbsolutePitch(_) => eat_atomic(parser),
            Token::MeasureBar() => eat_atomic(parser),
            _ => Err(SyntaxError::Unexpected("Illegal token in repeat block".to_string(),
                token.clone(), literal.clone())),
        }?;

        nodes.push(node);
    }

    Ok(nodes)
}

fn node_to_string(node: &ASTNode, level: u32) -> String
{
    let pad = (0..level*3).map(|_| " ").collect::<String>();

    match node
    {
        ASTNode::Tempo{literal, ..} => format!("{}[tempo] {}", pad, literal.literal),
        ASTNode::AbsolutePitch{literal, ..} => format!("{}[abspitch] {}", pad, literal.literal),
        ASTNode::Scale{literal, ..} => format!("{}[scale] {}", pad, literal.literal),
        ASTNode::Note{literal, ..} => format!("{}[note] {}", pad, literal.literal),
        ASTNode::Track{literal, ..} => format!("{}[track] {}", pad, literal.literal),
        ASTNode::BeatAssert{literal, ..} => format!("{}[beats] {}", pad, literal.literal),
        ASTNode::ScaleDegree{literal, ..}  => format!("{}[scaledeg] {}", pad, literal.literal),
        ASTNode::MeasureBar{literal, ..}  => format!("{}[mb] {}", pad, literal.literal),
        ASTNode::DynamicLevel{literal, ..}  => format!("{}[dyn] {}", pad, literal.literal),
        ASTNode::RepeatBlock{start_literal, end_literal, count, nodes} =>
        {
            let mut segments = vec![
                format!("{}[repeat] x{}", pad, count)];

            for n in nodes
            {
                segments.push(node_to_string(n, level + 1));
            }

            segments.join("\n")
        },
        ASTNode::Section{literal, name, nodes} =>
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

fn tree_to_string(tree: &AST) -> String
{
    let mut segments = vec!["[top]".to_string()];
    for node in &tree.nodes
    {
        segments.push(node_to_string(node, 1));
    }
    segments.push("[end]".to_string());
    segments.join("\n")
}

pub fn print_tree(tree: &AST)
{
    println!("{}", tree_to_string(tree))
}

pub fn print_parse_error(pe: &SyntaxError)
{
    match pe
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
           [repeat] x1
              [note] .
              [note] ./2
        [end]"});

    assert_ast_results("---BIG--- F3 . . F2 ./3 duh:3/2",
        indoc! {"
        [top]
           [section] ---BIG---
              [abspitch] F3
              [note] .
              [note] .
              [abspitch] F2
              [note] ./3
              [note] duh:3/2
        [end]"});
}
