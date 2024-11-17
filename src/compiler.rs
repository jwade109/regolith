use crate::types::{CompileResult, CompileError};
use crate::lexer::lex_markdown;
use crate::lexer::lex_multiline_string;
use crate::parser::parse_to_ast;
use crate::semantics::{Composition, do_semantics};
use crate::codegen::generate_mb_code;
use crate::moonbase::create_dir;

use std::path::Path;

pub enum CompileInput
{
    StringLiteral(String),
    Markdown(Path)
}

pub fn compile<>(input: &CompileInput, build_root: &Path) -> CompileResult<()>
{
    create_dir(&build_root)?;

    let build_name = match input
    {
        CompileInput::StringLiteral(s) =>
        {
            let hash = md5::compute(&s);
            format!("string-literal-{:x}", hash)
        },
        CompileInput::Markdown(p) =>
        {
            // let bytes = std::fs::read(p).unwrap();
            // let hash = md5::compute(&bytes);

            let err = || {
                CompileError::Generic("Bad filename")
            };

            let file_name = p.file_name().ok_or_else(err)?.to_str().ok_or_else(err)?;

            // format!("{}-{:x}", file_name, hash)
            file_name.to_string()
        }
    };

    let build_dir = build_root.join(build_name);

    println!("Build directory: {}", build_dir.display());

    create_dir(&build_dir)?;

    let cache_dir = build_root.join("cache");
    create_dir(&cache_dir)?;

    let tokens = match input
    {
        CompileInput::StringLiteral(s) =>
        {
            lex_multiline_string(s)
        },
        CompileInput::Markdown(p) =>
        {
            lex_markdown(p)
        }
    }?;

    let tree = parse_to_ast(&tokens)?;
    let comp = do_semantics(&tree)?;

    generate_mb_code(&comp, &cache_dir, &build_dir)?;

    println!("Done.\n");

    Ok(())
}
