use crate::lexer::{Literal, Token, lex_multiline_string, RegoNote, DynamicLevel};

pub struct Parser
{
    tokens: Vec<(Literal, Token)>
}

#[derive(Debug)]
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
        tone_id: u8,
        steps: Vec<u8>,
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
    pub nodes: Vec<ASTNode>,
}

type ParseResult<T> = Result<T, ParseError>;

impl Parser
{
    pub fn new(tokens: &Vec<(Literal, Token)>) -> Self
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

    pub fn parse_toplevel(&mut self) -> ParseResult<AST>
    {
        let mut nodes = vec![];

        while let Some((literal, token)) = self.peek()
        {
            let node = match token
            {
                Token::StartRepeat() => parse_repeat_block(self),
                Token::AbsolutePitch(_) => parse_absolute_pitch(self),
                Token::Tempo(_) => parse_tempo(self),
                Token::Scale(_) => parse_scale(self),
                Token::Track(_) => parse_track(self),
                Token::Note(_) => parse_note(self),
                Token::BeatAssert(_) => parse_beat_assertion(self),
                Token::Section(_) => parse_section(self),
                Token::ScaleDegree(_) => parse_scale_degree(self),
                Token::MeasureBar() => parse_measure_bar(self),
                Token::Dynamic(_) => parse_dynamic(self),
                Token::EndRepeat(_) => Err(ParseError::Unexpected(
                    "Unexpected repeat block terminator".to_string(),
                    token.clone(), literal.clone())),
            }?;

            nodes.push(node);
        }

        Ok(AST{ nodes })
    }
}

fn parse_scale_degree(parser: &mut Parser) -> ParseResult<ASTNode>
{
    if let Some((literal, token)) = parser.take()
    {
        if let Token::ScaleDegree(degree) = token
        {
            return Ok(ASTNode::ScaleDegree { literal, degree });
        }
        else
        {
            return Err(ParseError::Unexpected("Expected a scale degree".to_string(),
                token.clone(), literal.clone()));
        }
    }

    Err(ParseError::Generic("No token to take".to_string()))
}

fn parse_dynamic(parser: &mut Parser) -> ParseResult<ASTNode>
{
    if let Some((literal, token)) = parser.take()
    {
        if let Token::Dynamic(level) = token
        {
            return Ok(ASTNode::DynamicLevel { literal, level });
        }
        else
        {
            return Err(ParseError::Unexpected("Expected a dynamic declaration".to_string(),
                token.clone(), literal.clone()));
        }
    }

    Err(ParseError::Generic("No token to take".to_string()))
}

fn parse_measure_bar(parser: &mut Parser) -> ParseResult<ASTNode>
{
    if let Some((literal, token)) = parser.take()
    {
        if let Token::MeasureBar() = token
        {
            return Ok(ASTNode::MeasureBar { literal });
        }
        else
        {
            return Err(ParseError::Unexpected("Expected a measure bar".to_string(),
                token.clone(), literal.clone()));
        }
    }

    Err(ParseError::Generic("No token to take".to_string()))
}

fn parse_track(parser: &mut Parser) -> ParseResult<ASTNode>
{
    if let Some((literal, token)) = parser.take()
    {
        if let Token::Track(track_id) = token
        {
            return Ok(ASTNode::Track { literal, track_id });
        }
        else
        {
            return Err(ParseError::Unexpected("Expected a track directive".to_string(),
                token.clone(), literal.clone()));
        }
    }

    Err(ParseError::Generic("No token to take".to_string()))
}

fn parse_section(parser: &mut Parser) -> ParseResult<ASTNode>
{
    if let Some((literal, token)) = parser.take()
    {
        if let Token::Section(name) = token
        {
            return Ok(ASTNode::Section { literal, name });
        }
        else
        {
            return Err(ParseError::Unexpected("Expected a section header".to_string(),
                token.clone(), literal.clone()));
        }
    }

    Err(ParseError::Generic("No token to take".to_string()))
}

fn parse_scale(parser: &mut Parser) -> ParseResult<ASTNode>
{
    if let Some((literal, token)) = parser.take()
    {
        if let Token::Scale(s) = token
        {
            return Ok(ASTNode::Scale { literal, tone_id: s.tone_id, steps: s.steps });
        }
        else
        {
            return Err(ParseError::Unexpected("Expected a scale".to_string(),
                token.clone(), literal.clone()));
        }
    }

    Err(ParseError::Generic("No token to take".to_string()))
}

fn parse_tempo(parser: &mut Parser) -> ParseResult<ASTNode>
{
    if let Some((literal, token)) = parser.take()
    {
        if let Token::Tempo(t) = token
        {
            return Ok(ASTNode::Tempo { literal, tempo: t });
        }
        else
        {
            return Err(ParseError::Unexpected("Expected a tempo".to_string(),
                token.clone(), literal.clone()));
        }
    }

    Err(ParseError::Generic("No token to take".to_string()))
}

fn parse_beat_assertion(parser: &mut Parser) -> ParseResult<ASTNode>
{
    if let Some((literal, token)) = parser.take()
    {
        if let Token::BeatAssert(beats) = token
        {
            return Ok(ASTNode::BeatAssert { literal, beats });
        }
        else
        {
            return Err(ParseError::Unexpected("Expected a beat assertion".to_string(),
                token.clone(), literal.clone()));
        }
    }

    Err(ParseError::Generic("No token to take".to_string()))
}

fn parse_absolute_pitch(parser: &mut Parser) -> ParseResult<ASTNode>
{
    if let Some((literal, token)) = parser.take()
    {
        if let Token::AbsolutePitch(n) = token
        {
            return Ok(ASTNode::AbsolutePitch { literal, pitch: n });
        }
        else
        {
            return Err(ParseError::Unexpected("Expected a pitch".to_string(),
                token.clone(), literal.clone()));
        }
    }

    Err(ParseError::Generic("No token to take".to_string()))
}

fn parse_repeat_block(parser: &mut Parser) -> ParseResult<ASTNode>
{
    let (start_literal, _start_token) = parser.take()
        .ok_or(ParseError::Generic("Expected token at start of repeat block".to_string()))?;
    let nodes = parse_repeat_block_interior(parser)?;
    let (end_literal, end_token) = parser.take()
        .ok_or(ParseError::Generic("Expected token at end of repeat block".to_string()))?;
    if let Token::EndRepeat(count) = end_token
    {
        return Ok(ASTNode::RepeatBlock{ start_literal, end_literal, count, nodes });
    }
    Err(ParseError::Unexpected("Expected end repeat block token".to_string(),
        end_token.clone(), end_literal.clone()))
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
            Token::Note(_) => parse_note(parser),
            Token::AbsolutePitch(_) => parse_absolute_pitch(parser),
            Token::MeasureBar() => parse_measure_bar(parser),
            _ => Err(ParseError::Unexpected("Unexpected token in repeat block".to_string(),
                token.clone(), literal.clone())),
        }?;

        nodes.push(node);
    }

    Ok(nodes)
}

fn parse_note(parser: &mut Parser) -> ParseResult<ASTNode>
{
    if let Some((literal, token)) = parser.take()
    {
        if let Token::Note(n) = token
        {
            return Ok(ASTNode::Note { literal: literal.clone(), note: n.clone() })
        }
        else
        {
            return Err(ParseError::Unexpected("Expected a note".to_string(),
                token.clone(), literal.clone()));
        }
    }

    Err(ParseError::Generic("No token to take".to_string()))
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
        // ASTNode::Section{nodes} =>
        // {
        //     println!("SECTION");
        //     for n in nodes
        //     {
        //         print_node(n, level + 1);
        //     }
        //     println!("END SECTION");
        // }
        // _ => (),
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