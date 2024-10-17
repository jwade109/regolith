// use crate::parser::{ASTNode, AST};
// use crate::lexer::{DynamicLevel, Literal, Scale};

// #[derive(Debug, Clone)]
// pub enum SemanticError
// {
//     Generic(String),
//     GenericNode(String, ASTNode)
// }

// fn make_section(nodes: &Vec<ASTNode>) -> Result<(), SemanticError>
// {
//     let mut preamble = true;

//     let mut section_tempo: Option<u16> = None;
//     let mut section_scale: Option<Scale> = None;
//     let mut section_dynamic: Option<DynamicLevel> = None;

//     for node in nodes
//     {
//         if preamble
//         {
//             match node
//             {
//                 ASTNode::Tempo { literal, tempo } =>
//                 {
//                     if section_tempo.is_some()
//                     {
//                         return Err(SemanticError::GenericNode(
//                             "Cannot declare tempo twice in one section".to_string(), node.clone()));
//                     }
//                     section_tempo = Some(*tempo);
//                 }
//                 ASTNode::Scale { literal, scale } =>
//                 {
//                     if section_scale.is_some()
//                     {
//                         return Err(SemanticError::GenericNode(
//                             "Cannot declare scale twice in one section".to_string(), node.clone()));
//                     }
//                     section_scale = Some(scale.clone());
//                 },
//                 ASTNode::DynamicLevel { literal, level } =>
//                 {
//                     if section_dynamic.is_some()
//                     {
//                         return Err(SemanticError::GenericNode(
//                             "Cannot declare dynamic twice in one section".to_string(), node.clone()));
//                     }
//                     section_dynamic = Some(level.clone());
//                 }
//                 ASTNode::Note { .. } |
//                 ASTNode::MeasureBar { .. } |
//                 ASTNode::RepeatBlock { .. } |
//                 ASTNode::AbsolutePitch { .. } |
//                 ASTNode::ScaleDegree { .. } |
//                 ASTNode::BeatAssert { .. } |
//                 ASTNode::Track { .. } =>
//                 {
//                     preamble = false;
//                 },
//                 ASTNode::Section { .. } =>
//                 {
//                     return Err(SemanticError::GenericNode(
//                         "Illegal directive in section preamble".to_string(), node.clone()));
//                 },
//             }
//         }
//         else
//         {
//             match node
//             {
//                 ASTNode::Tempo { .. } |
//                 ASTNode::Scale { .. } |
//                 ASTNode::DynamicLevel { .. } =>
//                 {
//                     return Err(SemanticError::GenericNode(
//                         "Preamble directives are not allowed after the first staff element".to_string(),
//                         node.clone()));
//                 },
//                 ASTNode::Note { .. } |
//                 ASTNode::MeasureBar { .. } |
//                 ASTNode::RepeatBlock { .. } |
//                 ASTNode::AbsolutePitch { .. } |
//                 ASTNode::ScaleDegree { .. } |
//                 ASTNode::BeatAssert { .. } |
//                 ASTNode::Track { .. } =>
//                 {
//                     // neat
//                 },
//                 ASTNode::Section { .. } =>
//                 {
//                     return Err(SemanticError::GenericNode(
//                         "Illegal directive in section body".to_string(), node.clone()));
//                 },
//             }
//         }
//     }

//     println!("{:?} {:?} {:?}", section_tempo, section_scale, section_dynamic);

//     Ok(())
// }

// pub fn do_semantics(tree: &AST) -> Result<(), SemanticError>
// {
//     for node in &tree.nodes
//     {
//         match node
//         {
//             ASTNode::Section { literal, name, nodes } =>
//             {
//                 make_section(nodes)
//             }
//             _ => Ok(()),
//         }?;
//     }

//     Ok(())
// }
