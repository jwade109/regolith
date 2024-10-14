use crate::lexer::{Literal, Token, lex_multiline_string, RegoNote, DynamicLevel, Scale};

struct Parser
{
    tokens: Vec<(Literal, Token)>
}

#[derive(Debug)]
enum ASTNode
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
        // TODO
        // nodes: Vec<ASTNode>
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
pub enum ParseError
{
    Generic(String),
    Unexpected(String, Token, Literal),
}

#[derive(Debug)]
pub struct AST
{
    nodes: Vec<ASTNode>,
}

type ParseResult<T> = Result<T, ParseError>;

impl Parser
{
    fn new(tokens: &Vec<(Literal, Token)>) -> Self
    {
        Parser { tokens: tokens.iter().rev().map(|(l, t)| (l.clone(), t.clone())).collect() }
    }

    fn peek(&self) -> Option<&(Literal, Token)>
    {
        return self.tokens.last()
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
            Token::StartRepeat() => eat_repeat_block(&mut parser),
            Token::AbsolutePitch(_) => eat_atomic(&mut parser),
            Token::Tempo(_) => eat_atomic(&mut parser),
            Token::Scale(_) => eat_atomic(&mut parser),
            Token::Track(_) => eat_atomic(&mut parser),
            Token::Note(_) => eat_atomic(&mut parser),
            Token::BeatAssert(_) => eat_atomic(&mut parser),
            Token::Section(_) => eat_section(&mut parser),
            Token::ScaleDegree(_) => eat_atomic(&mut parser),
            Token::MeasureBar() => eat_atomic(&mut parser),
            Token::Dynamic(_) => eat_atomic(&mut parser),
            Token::EndRepeat(_) => Err(ParseError::Unexpected(
                "Unexpected repeat block terminator".to_string(),
                token.clone(), literal.clone())),
        }?;

        nodes.push(node);
    }

    Ok(AST{ nodes })
}

fn eat_section(parser: &mut Parser) -> ParseResult<ASTNode>
{
    if let Some((literal, token)) = parser.take()
    {
        return match token
        {
            Token::Section(name) => Ok(ASTNode::Section { literal, name }),
            _ => Err(ParseError::Unexpected("Expected a section header".to_string(), token, literal)),
        }
    }

    Err(ParseError::Generic("No token to take".to_string()))
}

fn eat_repeat_block(parser: &mut Parser) -> ParseResult<ASTNode>
{
    parser.take(); // TODO assert that this is [:

    let (start_literal, _start_token) = parser.take()
        .ok_or(ParseError::Generic("Expected token at start of repeat block".to_string()))?;
    let nodes = parse_repeat_block_interior(parser)?;
    let (end_literal, end_token) = parser.take()
        .ok_or(ParseError::Generic("Unterminated repeat block".to_string()))?;
    if let Token::EndRepeat(count) = end_token
    {
        return Ok(ASTNode::RepeatBlock{ start_literal, end_literal, count, nodes });
    }
    Err(ParseError::Unexpected("Expected end repeat block token".to_string(),
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
        Token::Section(name) => Some(ASTNode::Section { literal, name }),
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
            return Err(ParseError::Unexpected(
                "Expected an atomic token, but got".to_string(), token, literal));
        }
    }

    Err(ParseError::Generic("Expected an atomic token, but nothing left".to_string()))
}

fn parse_repeat_block_interior(parser: &mut Parser) -> ParseResult<Vec<ASTNode>>
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
            _ => Err(ParseError::Unexpected("Illegal token in repeat block".to_string(),
                token.clone(), literal.clone())),
        }?;

        nodes.push(node);
    }

    Ok(nodes)
}

fn print_node(node: &ASTNode, level: u32)
{
    let pad = (0..level*3).map(|_| " ").collect::<String>();

    match node
    {
        ASTNode::Tempo{literal, ..} => println!("{}[tempo] {}", pad, literal.literal),
        ASTNode::AbsolutePitch{literal, ..} => println!("{}[abspitch] {}", pad, literal.literal),
        ASTNode::Scale{literal, ..} => println!("{}[scale] {}", pad, literal.literal),
        ASTNode::Note{literal, ..} => println!("{}[note] {}", pad, literal.literal),
        ASTNode::Track{literal, ..} => println!("{}[track] {}", pad, literal.literal),
        ASTNode::BeatAssert{literal, ..} => println!("{}[beats] {}", pad, literal.literal),
        ASTNode::Section{literal, ..}  => println!("{}[section] {}", pad, literal.literal),
        ASTNode::ScaleDegree{literal, ..}  => println!("{}[scaledeg] {}", pad, literal.literal),
        ASTNode::MeasureBar{literal, ..}  => println!("{}[mb] {}", pad, literal.literal),
        ASTNode::DynamicLevel{literal, ..}  => println!("{}[dyn] {}", pad, literal.literal),
        ASTNode::RepeatBlock{start_literal, end_literal, count, nodes} =>
        {
            println!("{}[oprpt] {}", pad, start_literal.literal);
            for n in nodes
            {
                print_node(n, level + 1);
            }
            println!("{}[clrpt] {} ({})", pad, end_literal.literal, count);
        }
    }
}

pub fn print_tree(tree: &AST)
{
    println!("[top]");
    for node in &tree.nodes
    {
        print_node(&node, 1);
    }
}

pub fn print_parse_error(pe: &ParseError)
{
    match pe
    {
        ParseError::Generic(msg) =>
        {
            println!("\n  Generic parse error: {}\n", msg);
        },
        ParseError::Unexpected(msg, token, literal) =>
        {
            println!("\n  Unexpected token: {} - {:?}, \"{}\", line {}, col {}\n",
                msg, token, literal.literal, literal.lineno, literal.colno);
        },
    }
}
