use crate::{Delimiters, Error, Result, Span};

#[derive(Debug, Clone)]
pub struct Lexer<'e, 't> {
    delims: &'e Delimiters<'e>,
    pub source: &'t str,
    state: State,
    cursor: usize,
    next: Option<(Token, Span)>,
}

#[derive(Debug, Clone)]
enum State {
    Template,
    InBlock { begin: Span, end: Token },
}

/// The type of token yielded by the lexer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    /// Raw template
    Raw,
    /// A begin expression tag, e.g. `{{`
    BeginExpr,
    /// A end expression tag, e.g. `}}`
    EndExpr,
    /// A begin block tag, e.g. `{%`
    BeginBlock,
    /// An end block tag, e.g. `%}`
    EndBlock,
    /// `.`
    Period,
    /// `|`
    Pipe,
    /// A sequence of tab (0x09) and/or spaces (0x20)
    Whitespace,
    /// An attribute, variable, or possible literal
    Ident,
}

impl Token {
    pub fn human(&self) -> &'static str {
        match self {
            Self::Raw => "raw template",
            Self::BeginExpr => "begin tag",
            Self::EndExpr => "end tag",
            Self::BeginBlock => "begin tag",
            Self::EndBlock => "end tag",
            Self::Period => "a period",
            Self::Pipe => "a pipe",
            Self::Whitespace => "whitespace",
            Self::Ident => "an identifier",
        }
    }

    fn pair(&self) -> Self {
        match self {
            Self::BeginExpr => Self::EndExpr,
            Self::EndExpr => Self::BeginExpr,
            Self::BeginBlock => Self::EndBlock,
            Self::EndBlock => Self::BeginBlock,
            _ => panic!("not a tag"),
        }
    }

    fn is_begin_tag(&self) -> bool {
        matches!(self, Self::BeginExpr | Self::BeginBlock)
    }

    pub fn is_whitespace(&self) -> bool {
        matches!(self, Self::Whitespace)
    }
}

impl<'e, 't> Lexer<'e, 't> {
    pub fn new(source: &'t str, delims: &'e Delimiters<'e>) -> Self {
        Self {
            delims,
            source,
            state: State::Template,
            cursor: 0,
            next: None,
        }
    }

    pub fn next(&mut self) -> Result<Option<(Token, Span)>> {
        loop {
            match self.lex()? {
                Some((tk, sp)) if !tk.is_whitespace() => return Ok(Some((tk, sp))),
                None => return Ok(None),
                _ => continue,
            }
        }
    }

    fn lex(&mut self) -> Result<Option<(Token, Span)>> {
        if let Some(next) = self.next.take() {
            return Ok(Some(next));
        }

        let i = self.cursor;

        if self.source[i..].is_empty() {
            return Ok(None);
        }

        // Looks for a tag at position `i`
        let try_lex_tag = |i| {
            let mut result = None;
            let bytes = &self.source.as_bytes()[i..];
            for (tag, needle) in [
                (Token::BeginExpr, self.delims.begin_expr),
                (Token::EndExpr, self.delims.end_expr),
                (Token::BeginBlock, self.delims.begin_block),
                (Token::EndBlock, self.delims.end_block),
            ] {
                let len = needle.len();
                if bytes.len() >= len && &bytes[..len] == needle.as_bytes() {
                    let update = match result {
                        None => true,
                        Some((_, curr)) if len > curr => true,
                        Some(_) => false,
                    };
                    if update {
                        result = Some((tag, len));
                    }
                }
            }
            result.map(|(tag, len)| (tag, i + len))
        };

        match self.state {
            State::Template => {
                let mut lex = |tk: Token, m, n| match tk {
                    tk if tk.is_begin_tag() => {
                        let begin = Span::from(m..n);
                        let end = tk.pair();
                        self.cursor = n;
                        self.state = State::InBlock { begin, end };
                        Ok(Some((tk, begin)))
                    }
                    _ => Err(Error::span(
                        format!("unexpected {}", tk.human()),
                        self.source,
                        m..n,
                    )),
                };

                match (i..self.source.len())
                    .find_map(|j| try_lex_tag(j).map(|(tag, k)| (tag, j, k)))
                {
                    Some((tag, j, k)) if i == j => lex(tag, j, k),
                    Some((tag, j, k)) => {
                        let now = (Token::Raw, Span::from(i..j));
                        self.next = lex(tag, j, k)?;
                        Ok(Some(now))
                    }
                    None => {
                        let j = self.source.len();
                        self.cursor = j;
                        Ok(Some((Token::Raw, Span::from(i..j))))
                    }
                }
            }

            State::InBlock { begin, end } => {
                let mut iter = self.source[i..].char_indices().map(|(d, c)| (i + d, c));
                let (i, c) = iter.next().unwrap();
                let (token, j) = match c {
                    // Single character to token mappings
                    '.' => (Token::Period, i + 1),
                    '|' => (Token::Pipe, i + 1),

                    // Multi character tokens with a distinct starting character
                    c if is_whitespace(c) => self.lex_while(iter, Token::Whitespace, is_whitespace),
                    c if is_ident(c) => self.lex_while(iter, Token::Ident, is_ident),

                    // Any other character
                    c => {
                        match try_lex_tag(i) {
                            // A matching end tag
                            Some((tk, j)) if tk == end => {
                                self.state = State::Template;
                                (tk, j)
                            }
                            Some((tk, _)) if tk.is_begin_tag() => {
                                return Err(Error::span(
                                    format!("unclosed {}", end.pair().human()),
                                    self.source,
                                    begin,
                                ))
                            }
                            Some((tk, j)) => {
                                return Err(Error::span(
                                    format!("unexpected {}", tk.human()),
                                    self.source,
                                    i..j,
                                ))
                            }
                            None => {
                                return Err(Error::span(
                                    "unexpected character",
                                    self.source,
                                    i..(i + c.len_utf8()),
                                ))
                            }
                        }
                    }
                };
                self.cursor = j;
                Ok(Some((token, Span::from(i..j))))
            }
        }
    }

    fn lex_while<I, P>(&mut self, mut iter: I, tk: Token, pred: P) -> (Token, usize)
    where
        I: Iterator<Item = (usize, char)> + Clone,
        P: Fn(char) -> bool,
    {
        loop {
            match iter.clone().next() {
                Some((_, c)) if pred(c) => {
                    iter.next().unwrap();
                }
                Some((j, _)) => return (tk, j),
                None => return (tk, self.source.len()),
            }
        }
    }
}

fn is_whitespace(c: char) -> bool {
    matches!(c, '\t' | ' ')
}

fn is_ident(c: char) -> bool {
    matches!(c, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn lex_empty() {
        let tokens = lex("").unwrap();
        assert_eq!(tokens, []);
    }

    #[test]
    fn lex_raw() {
        let tokens = lex("lorem ipsum").unwrap();
        assert_eq!(tokens, [(Token::Raw, "lorem ipsum")]);
    }

    #[test]
    fn lex_begin_block() {
        let tokens = lex("lorem ipsum {%").unwrap();
        assert_eq!(
            tokens,
            [(Token::Raw, "lorem ipsum "), (Token::BeginBlock, "{%"),]
        );
    }

    #[test]
    fn lex_empty_block() {
        let tokens = lex("lorem ipsum {%%}").unwrap();
        assert_eq!(
            tokens,
            [
                (Token::Raw, "lorem ipsum "),
                (Token::BeginBlock, "{%"),
                (Token::EndBlock, "%}"),
            ]
        );
    }

    #[test]
    fn lex_begin_expr() {
        let tokens = lex("lorem ipsum {{").unwrap();
        assert_eq!(
            tokens,
            [(Token::Raw, "lorem ipsum "), (Token::BeginExpr, "{{"),]
        );
    }

    #[test]
    fn lex_empty_expr() {
        let tokens = lex("lorem ipsum {{}}").unwrap();
        assert_eq!(
            tokens,
            [
                (Token::Raw, "lorem ipsum "),
                (Token::BeginExpr, "{{"),
                (Token::EndExpr, "}}"),
            ]
        );
    }

    #[test]
    fn lex_expr() {
        let tokens = lex("lorem ipsum {{ .|\t aZ_0 }} dolor sit amet").unwrap();
        assert_eq!(
            tokens,
            [
                (Token::Raw, "lorem ipsum "),
                (Token::BeginExpr, "{{"),
                (Token::Whitespace, " "),
                (Token::Period, "."),
                (Token::Pipe, "|"),
                (Token::Whitespace, "\t "),
                (Token::Ident, "aZ_0"),
                (Token::Whitespace, " "),
                (Token::EndExpr, "}}"),
                (Token::Raw, " dolor sit amet"),
            ]
        );
    }

    #[test]
    fn lex_block_and_expr() {
        let tokens =
            lex("{% if cond %} lorem ipsum {{ path.segment }} dolor sit amet {% end %}").unwrap();
        assert_eq!(
            tokens,
            [
                (Token::BeginBlock, "{%"),
                (Token::Whitespace, " "),
                (Token::Ident, "if"),
                (Token::Whitespace, " "),
                (Token::Ident, "cond"),
                (Token::Whitespace, " "),
                (Token::EndBlock, "%}"),
                (Token::Raw, " lorem ipsum "),
                (Token::BeginExpr, "{{"),
                (Token::Whitespace, " "),
                (Token::Ident, "path"),
                (Token::Period, "."),
                (Token::Ident, "segment"),
                (Token::Whitespace, " "),
                (Token::EndExpr, "}}"),
                (Token::Raw, " dolor sit amet "),
                (Token::BeginBlock, "{%"),
                (Token::Whitespace, " "),
                (Token::Ident, "end"),
                (Token::Whitespace, " "),
                (Token::EndBlock, "%}"),
            ]
        );
    }

    #[test]
    fn lex_unexpected_end_expr() {
        let err = lex("lorem ipsum }} dolor sit amet").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem ipsum }} dolor sit amet
   |             ^^ unexpected end tag
"
        );
    }

    #[test]
    fn lex_unexpected_end_block() {
        let err = lex("lorem ipsum %} dolor sit amet").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem ipsum %} dolor sit amet
   |             ^^ unexpected end tag
"
        );
    }

    #[test]
    fn lex_unclosed_begin_expr() {
        let err = lex("lorem ipsum {{ {{ dolor sit amet").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem ipsum {{ {{ dolor sit amet
   |             ^^ unclosed begin tag
"
        );
    }

    #[test]
    fn lex_unclosed_begin_block() {
        let err = lex("lorem ipsum {% {{ dolor sit amet").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem ipsum {% {{ dolor sit amet
   |             ^^ unclosed begin tag
"
        );
    }

    #[test]
    fn lex_unexpected_end_tag() {
        let err = lex("lorem ipsum {{ %} dolor sit amet").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem ipsum {{ %} dolor sit amet
   |                ^^ unexpected end tag
"
        );
    }

    #[test]
    fn lex_unexpected_character() {
        let err = lex("lorem ipsum {{ ✨ }} dolor sit amet").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem ipsum {{ ✨ }} dolor sit amet
   |                ^^ unexpected character
"
        );
    }

    #[track_caller]
    fn lex(source: &str) -> Result<Vec<(Token, &str)>> {
        let delims = Delimiters::default();
        let mut lexer = Lexer::new(source, &delims);
        let mut tokens = Vec::new();
        while let Some((tk, sp)) = lexer.lex()? {
            tokens.push((tk, &source[sp]));
        }
        for _ in 0..3 {
            assert!(lexer.lex().unwrap().is_none());
        }
        Ok(tokens)
    }
}
