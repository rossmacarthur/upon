use std::fmt::Write;

use upon::Value;

fn main() -> upon::Result<()> {
    let mut engine = upon::Engine::new();

    // Overrides the default formatter, so that by default all values are
    // escaped for HTML.
    engine.set_default_formatter(&escape_html);

    // Adds a custom formatter that can be manually specified when rendering
    // in order to not escape anything.
    engine.add_formatter("unescape", upon::fmt::default);

    let template = engine.compile(
        "
Escaped: {{ value }}

Unescaped: {{ value | unescape }}
",
    )?;

    // A value containing something that would be escaped.
    let ctx = upon::value! {
        value: "'<this>' & \"<that>\"",
    };

    println!("{}", template.render(ctx)?);
    Ok(())
}

/// Custom implementation copied from [rustdoc] but this could also be
/// implemented using a crate from Crates.io for example.
///
/// [rustdoc]: https://github.com/rust-lang/rust/blob/4596f4f8b565bdd02d3b99d1ab12ff09146a93de/src/librustdoc/html/escape.rs
fn escape_html(f: &mut upon::fmt::Formatter<'_>, value: &upon::Value) -> upon::fmt::Result {
    let s = match value {
        Value::String(s) => s,
        value => {
            // Fallback to default formatter
            return upon::fmt::default(f, value);
        }
    };

    let mut last = 0;
    for (i, byte) in s.bytes().enumerate() {
        match byte {
            b'<' | b'>' | b'&' | b'\'' | b'"' => {
                f.write_str(&s[last..i])?;
                let s = match byte {
                    b'>' => "&gt;",
                    b'<' => "&lt;",
                    b'&' => "&amp;",
                    b'\'' => "&#39;",
                    b'"' => "&quot;",
                    _ => unreachable!(),
                };
                f.write_str(s)?;
                last = i + 1;
            }
            _ => {}
        }
    }
    if last < s.len() {
        f.write_str(&s[last..])?;
    }
    Ok(())
}
