//! A little tool to generate README.md from lib.rs

use std::borrow::Borrow;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use pulldown_cmark_to_cmark::{cmark_resume_with_options, Options as Options2};

const WORKSPACE_DIR: &str = env!("CARGO_WORKSPACE_DIR");

fn main() -> Result<()> {
    let engine = upon::Engine::new();
    update_readme(&engine)?;
    update_syntax(&engine)?;
    Ok(())
}

fn update_readme(engine: &upon::Engine) -> Result<()> {
    let readme = PathBuf::from_iter([WORKSPACE_DIR, "README.md"]);
    let librs = PathBuf::from_iter([WORKSPACE_DIR, "src", "lib.rs"]);
    let text = get_module_comment(&librs)?;
    let docs = reformat(&text)?;
    let result = engine
        .compile(include_str!("README_TEMPLATE.md"))?
        .render(upon::value! {
            docs: docs
        })?;
    fs::write(&readme, result)?;
    Ok(())
}

fn update_syntax(engine: &upon::Engine) -> Result<()> {
    let syntax = PathBuf::from_iter([WORKSPACE_DIR, "SYNTAX.md"]);
    let syntaxrs = PathBuf::from_iter([WORKSPACE_DIR, "src", "syntax.rs"]);
    let text = get_module_comment(&syntaxrs)?;
    let docs = reformat(&text)?;
    let i = docs.find(|c| c == '\n').unwrap();

    let result = engine
        .compile(include_str!("SYNTAX_TEMPLATE.md"))?
        .render(upon::value! {
            docs: &docs[i+2..]
        })?;
    fs::write(&syntax, result)?;
    Ok(())
}

fn get_module_comment(path: &Path) -> Result<String> {
    let contents = fs::read_to_string(path)?;
    let lines: Vec<_> = contents
        .lines()
        .take_while(|line| line.starts_with("//!"))
        .map(|line| line.trim_start_matches("//! ").trim_start_matches("//!"))
        .collect();
    Ok(lines.join("\n"))
}

/// Reformat a Markdown file and increase the heading level.
fn reformat(text: &str) -> Result<String> {
    let mut events = Vec::from_iter(Parser::new_ext(text, Options::all()));
    events = fix_headings(events);
    events = fix_code_blocks(events);
    events = fix_links(events);
    to_cmark(events)
}

/// Increases each heading level by one.
fn fix_headings(events: Vec<Event>) -> Vec<Event> {
    let mut iter = events.into_iter();
    let mut events = Vec::new();
    while let Some(event) = iter.next() {
        match event {
            Event::Start(Tag::Heading(level, frag, classes)) => {
                let tag = Tag::Heading((level as usize + 1).try_into().unwrap(), frag, classes);
                events.push(Event::Start(tag.clone()));
                loop {
                    match iter.next().unwrap() {
                        Event::End(Tag::Heading(..)) => break,
                        event => events.push(event),
                    }
                }
                events.push(Event::End(tag));
            }
            event => events.push(event),
        }
    }
    events
}

/// Fixes intra-doc links.
fn fix_links(events: Vec<Event>) -> Vec<Event> {
    let mut iter = events.into_iter().peekable();
    let mut events = Vec::new();
    while let Some(event) = iter.next() {
        match event {
            Event::Text(text) if text.as_ref() == "[" => {
                loop {
                    match iter.next().unwrap() {
                        Event::Text(text) if text.as_ref() == "]" => break,
                        event => events.push(event),
                    }
                }

                match iter.peek() {
                    Some(Event::Text(text)) if text.as_ref() == "[" => {
                        iter.next().unwrap();
                        loop {
                            match iter.next().unwrap() {
                                Event::Text(text) if text.as_ref() == "]" => break,
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            event => events.push(event),
        }
    }
    events
}

/// Fixes code blocks.
fn fix_code_blocks(events: Vec<Event>) -> Vec<Event> {
    let mut iter = events.into_iter();
    let mut events = Vec::new();
    while let Some(event) = iter.next() {
        match event {
            Event::Start(Tag::CodeBlock(kind)) if is_rust(&kind) => {
                let tag = Tag::CodeBlock(fix_code_block_kind(kind));

                let code = match iter.next().unwrap() {
                    Event::Text(code) => fix_code_block(code),
                    event => panic!("unexpected event `{:?}`", event),
                };

                match iter.next().unwrap() {
                    Event::End(Tag::CodeBlock(_)) => {}
                    event => panic!("unexpected event `{:?}`", event),
                };

                events.push(Event::Start(tag.clone()));
                events.push(Event::Text(code));
                events.push(Event::End(tag));
            }
            event => events.push(event),
        }
    }
    events
}

/// Checks whether a code block is a Rust one.
fn is_rust(kind: &CodeBlockKind) -> bool {
    match kind {
        CodeBlockKind::Fenced(attr) if attr.is_empty() => true,
        CodeBlockKind::Fenced(attr) if attr.as_ref() == "rust" => true,
        CodeBlockKind::Fenced(_) => false,
        _ => true,
    }
}

/// Makes empty code blocks `rust` code blocks.
fn fix_code_block_kind(kind: CodeBlockKind) -> CodeBlockKind {
    match kind {
        CodeBlockKind::Fenced(attr) if attr.is_empty() => {
            CodeBlockKind::Fenced(CowStr::Borrowed("rust"))
        }
        kind => kind,
    }
}

/// Rewrites code blocks to exclude `#` prefixed lines.
fn fix_code_block(code: CowStr) -> CowStr {
    let mut result = String::new();
    for line in code
        .lines()
        .filter(|line| !line.trim().starts_with("# ") && *line != "#")
    {
        result.push_str(line);
        result.push('\n');
    }
    CowStr::Boxed(result.into_boxed_str())
}

/// Render Markdown events as Markdown.
fn to_cmark<'a, I, E>(events: I) -> Result<String>
where
    I: IntoIterator<Item = E>,
    E: Borrow<Event<'a>>,
{
    let mut buf = String::new();
    let opts = Options2 {
        code_block_token_count: 3,
        list_token: '-',
        ..Default::default()
    };
    cmark_resume_with_options(events.into_iter(), &mut buf, None, opts)?.finalize(&mut buf)?;
    Ok(buf)
}
