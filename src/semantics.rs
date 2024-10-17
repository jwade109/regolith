// use crate::parser::{StaffNode, AST};
// use crate::lexer::{DynamicLevel, Literal, Scale};

// #[derive(Debug, Clone)]
// pub enum SemanticError
// {
//     Generic(String),
//     GenericNode(String, StaffNode)
// }

// fn make_section(nodes: &Vec<StaffNode>) -> Result<(), SemanticError>
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
//                 StaffNode::Tempo { literal, tempo } =>
//                 {
//                     if section_tempo.is_some()
//                     {
//                         return Err(SemanticError::GenericNode(
//                             "Cannot declare tempo twice in one section".to_string(), node.clone()));
//                     }
//                     section_tempo = Some(*tempo);
//                 }
//                 StaffNode::Scale { literal, scale } =>
//                 {
//                     if section_scale.is_some()
//                     {
//                         return Err(SemanticError::GenericNode(
//                             "Cannot declare scale twice in one section".to_string(), node.clone()));
//                     }
//                     section_scale = Some(scale.clone());
//                 },
//                 StaffNode::DynamicLevel { literal, level } =>
//                 {
//                     if section_dynamic.is_some()
//                     {
//                         return Err(SemanticError::GenericNode(
//                             "Cannot declare dynamic twice in one section".to_string(), node.clone()));
//                     }
//                     section_dynamic = Some(level.clone());
//                 }
//                 StaffNode::Note { .. } |
//                 StaffNode::MeasureBar { .. } |
//                 StaffNode::RepeatBlock { .. } |
//                 StaffNode::AbsolutePitch { .. } |
//                 StaffNode::ScaleDegree { .. } |
//                 StaffNode::BeatAssert { .. } |
//                 StaffNode::Track { .. } =>
//                 {
//                     preamble = false;
//                 },
//                 StaffNode::Section { .. } =>
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
//                 StaffNode::Tempo { .. } |
//                 StaffNode::Scale { .. } |
//                 StaffNode::DynamicLevel { .. } =>
//                 {
//                     return Err(SemanticError::GenericNode(
//                         "Preamble directives are not allowed after the first staff element".to_string(),
//                         node.clone()));
//                 },
//                 StaffNode::Note { .. } |
//                 StaffNode::MeasureBar { .. } |
//                 StaffNode::RepeatBlock { .. } |
//                 StaffNode::AbsolutePitch { .. } |
//                 StaffNode::ScaleDegree { .. } |
//                 StaffNode::BeatAssert { .. } |
//                 StaffNode::Track { .. } =>
//                 {
//                     // neat
//                 },
//                 StaffNode::Section { .. } =>
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
//             StaffNode::Section { literal, name, nodes } =>
//             {
//                 make_section(nodes)
//             }
//             _ => Ok(()),
//         }?;
//     }

//     Ok(())
// }
