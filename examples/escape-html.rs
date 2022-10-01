//! Demonstrates how you can override the default formatter to escape strings
//! for HTML and add a "unescape" formatter for cases where you don't want to
//! escape.

use std::fmt::Write;

use upon::{Engine, Formatter, Result, Value};

fn main() -> Result<()> {
    let mut engine = Engine::new();
    engine.set_default_formatter(&escape_html);
    engine.add_formatter("unescape", upon::format);

    let result = engine
        .compile(
            "
Escaped: {{ value }}

Unescaped: {{ value | unescape }}
",
        )?
        .render(upon::value! {
            value: "'<this>' & \"<that>\"",
        })?;

    println!("{}", result);
    Ok(())
}

fn escape_html(f: &mut Formatter<'_>, value: &Value) -> Result<()> {
    let s = match value {
        Value::String(s) => s,
        value => {
            // Fallback to default formatter
            return upon::format(f, value);
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
