use crate::compile::parse::Keyword;
use crate::syntax;
use crate::types::span::Span;
use crate::{Engine, Error, Result};

/// A lexer that tokenizes the template source into distinct chunks so that the
/// parser doesn't have to operate on raw text.
///
/// The lexer is implemented as a fallible iterator. The parser should
/// repeatedly call the [`.next()?`][Lexer::next] method to return the next
/// non-whitespace token until [`None`] is returned.
#[cfg_attr(test, derive(Debug))]
pub struct Lexer<'engine, 'source> {
    /// A reference to the engine containing the syntax searcher.
    engine: &'engine Engine<'engine>,

    /// The original template source.
    pub source: &'source str,

    /// A cursor over the template source.
    cursor: usize,

    /// The current state of the lexer.
    state: State,

    /// Whether to left trim the next raw token.
    left_trim: bool,

    /// A buffer to store the next token.
    next: Option<(Token, Span)>,
}

/// The state of the lexer.
///
/// The lexer requires state because the tokenization is different when
/// tokenizing text between expression and block syntax, e.g. `{{ expr }}`,
/// `{% if cond %}`.
#[cfg_attr(test, derive(Debug))]
enum State {
    /// Within raw template.
    Template,

    /// Between expression or block tags.
    Block {
        /// The span of the begin tag.
        begin: Span,
        /// The end token we are expecting.
        end: Token,
    },

    /// Between comment tags.
    Comment {
        /// The span of the begin tag.
        begin: Span,
        /// The end token we are expecting.
        end: Token,
    },
}

/// The unit yielded by the lexer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    /// Raw template
    Raw,
    /// Begin expression tag, e.g. `{{`
    BeginExpr,
    /// End expression tag, e.g. `}}`
    EndExpr,
    /// Begin block tag, e.g. `{%`
    BeginBlock,
    /// End block tag, e.g. `%}`
    EndBlock,
    /// Begin block tag, e.g. `{#`
    BeginComment,
    /// End block tag, e.g. `#}`
    EndComment,
    /// `.`
    Period,
    /// `|`
    Pipe,
    /// `,`
    Comma,
    /// Sequence of tab (0x09) and/or spaces (0x20)
    Whitespace,
    /// An attribute or variable
    Ident,
    /// A keyword like `if` or `for`
    Keyword,
}

impl<'engine, 'source> Lexer<'engine, 'source> {
    /// Construct a new lexer.
    pub fn new(engine: &'engine Engine<'engine>, source: &'source str) -> Self {
        Self {
            engine,
            source,
            cursor: 0,
            state: State::Template,
            left_trim: false,
            next: None,
        }
    }

    /// Returns the next non-whitespace token and its span.
    pub fn next(&mut self) -> Result<Option<(Token, Span)>> {
        loop {
            match self.lex()? {
                Some((tk, sp)) if !tk.is_whitespace() => return Ok(Some((tk, sp))),
                None => return Ok(None),
                _ => continue,
            }
        }
    }

    /// Returns the next token and span.
    fn lex(&mut self) -> Result<Option<(Token, Span)>> {
        if let Some(next) = self.next.take() {
            return Ok(Some(next));
        }

        let i = self.cursor;

        if self.source[i..].is_empty() {
            return Ok(None);
        }

        match self.state {
            State::Template => self.lex_template(i),
            State::Block { begin, end } => self.lex_block(begin, end, i),
            State::Comment { begin, end } => self.lex_comment(begin, end, i),
        }
    }

    fn lex_template(&mut self, i: usize) -> Result<Option<(Token, Span)>> {
        // We are within raw template, that means all we have to do is
        // find the next begin tag from `i` and and any relevant cursor
        // indexes. The following diagram helps describe the variable
        // naming.
        //
        // xxxxxxx{{xxxxxxxxx
        //    ^   ^ ^
        //    i   j k

        let mut trim_raw_token = |mut i, mut j, right_trim| {
            if right_trim {
                j = self.source[..j].trim_end().len();
            }
            if self.left_trim {
                self.left_trim = false;
                let s = &self.source[i..j];
                i += s.len() - s.trim_start().len();
            }
            Ok(Some((Token::Raw, Span::from(i..j))))
        };

        match self.engine.searcher.find_at(&self.source, i) {
            Some((kind, j, k)) => {
                let (tk, trim) = Token::from_kind(kind);

                if !tk.is_begin_tag() {
                    return Err(self.err_unexpected_token(tk, j..k));
                }

                // Updates the current lexer cursor and state and
                // returns the token and span.
                let mut lex = |m, n| {
                    let begin = Span::from(m..n);
                    let end = tk.pair();
                    self.cursor = n;
                    self.state = if tk.is_begin_comment() {
                        State::Comment { begin, end }
                    } else {
                        State::Block { begin, end }
                    };
                    Ok(Some((tk, begin)))
                };

                if i == j {
                    // The current cursor is exactly at the token.
                    lex(j, k)
                } else {
                    // We must first emit the raw token, so we store the
                    // begin tag token in the `next` buffer.
                    self.next = lex(j, k)?;
                    trim_raw_token(i, j, trim)
                }
            }
            None => {
                let j = self.source.len();
                self.cursor = j;
                trim_raw_token(i, j, false)
            }
        }
    }

    fn lex_block(&mut self, begin: Span, end: Token, i: usize) -> Result<Option<(Token, Span)>> {
        // We are between two tags {{ ... }} or {% ... %} that means we
        // must parse template syntax relevant tokens and also lookout
        // for the corresponding end tag `end`.

        // We iterate over chars because that is nicer than operating on
        // raw bytes. The map call here fixes the index to be relative to the
        // actual template source.
        let mut iter = self.source[i..].char_indices().map(|(d, c)| (i + d, c));

        // We can `.unwrap()` since we've already checked that there is
        // more text remaining.
        let (i, c) = iter.next().unwrap();

        let (tk, j) = match c {
            // Single character to token mappings.
            '.' => (Token::Period, i + 1),
            '|' => (Token::Pipe, i + 1),
            ',' => (Token::Comma, i + 1),

            // Multi-character tokens with a distinct start character.
            c if is_whitespace(c) => self.lex_whitespace(iter),
            c if is_ident(c) => self.lex_ident_or_keyword(iter, i),

            // Any other character...
            _ => {
                match self.engine.searcher.starts_with(&self.source, i) {
                    Some((kind, j)) => {
                        let (tk, trim) = Token::from_kind(kind);

                        if tk.is_begin_tag() {
                            return Err(self.err_unclosed(begin, end));
                        }
                        if tk != end {
                            return Err(self.err_unexpected_token(tk, i..j));
                        }

                        // A matching end tag! Update the state and
                        // return the token.
                        self.state = State::Template;
                        self.left_trim = trim;
                        (tk, j)
                    }
                    None => {
                        return Err(self.err_unexpected_character(i..(i + c.len_utf8())));
                    }
                }
            }
        };

        // Finally, we need to update the cursor.
        self.cursor = j;

        Ok(Some((tk, Span::from(i..j))))
    }

    fn lex_comment(&mut self, begin: Span, end: Token, i: usize) -> Result<Option<(Token, Span)>> {
        // We are between two comment tags {# ... #}, that means all we
        // have to do is find the corresponding end tag. The following
        // diagram helps describe the variable naming.
        //
        // x{#cccccc#}xxxxxx
        //    ^     ^ ^
        //    i     j k

        match self.engine.searcher.find_at(&self.source, i) {
            Some((kind, j, k)) => {
                let (tk, trim) = Token::from_kind(kind);

                if tk.is_begin_tag() {
                    return Err(self.err_unclosed(begin, end));
                }
                if tk != end {
                    return Err(self.err_unexpected_token(tk, j..k));
                }

                // Updates the current lexer cursor and state and returns the
                // token and span.
                let mut lex = |m, n| {
                    self.cursor = n;
                    self.state = State::Template;
                    self.left_trim = trim;
                    let end = Span::from(m..n);
                    Ok(Some((tk, end)))
                };

                if i == j {
                    // The current cursor is exactly at the token.
                    lex(j, k)
                } else {
                    // We must first emit the raw token, so we store the end tag
                    // token in the `next` buffer.
                    self.next = lex(j, k)?;
                    Ok(Some((Token::Raw, Span::from(i..j))))
                }
            }
            None => {
                let j = self.source.len();
                self.cursor = j;
                Ok(Some((Token::Raw, Span::from(i..j))))
            }
        }
    }

    fn lex_whitespace<I>(&mut self, iter: I) -> (Token, usize)
    where
        I: Iterator<Item = (usize, char)> + Clone,
    {
        (Token::Whitespace, self.lex_while(iter, is_whitespace))
    }

    fn lex_ident_or_keyword<I>(&mut self, iter: I, i: usize) -> (Token, usize)
    where
        I: Iterator<Item = (usize, char)> + Clone,
    {
        let j = self.lex_while(iter, is_ident);
        let tk = match Keyword::all().contains(&&self.source[i..j]) {
            true => Token::Keyword,
            false => Token::Ident,
        };
        (tk, j)
    }

    fn lex_while<I, P>(&mut self, mut iter: I, pred: P) -> usize
    where
        I: Iterator<Item = (usize, char)> + Clone,
        P: Fn(char) -> bool,
    {
        loop {
            match iter.clone().next() {
                Some((_, c)) if pred(c) => {
                    iter.next().unwrap();
                }
                Some((j, _)) => return j,
                None => return self.source.len(),
            }
        }
    }

    fn err_unclosed(&self, begin: Span, end: Token) -> Error {
        Error::new(
            format!("unclosed {}", end.pair().human()),
            self.source,
            begin,
        )
    }

    fn err_unexpected_token(&self, tk: Token, span: impl Into<Span>) -> Error {
        Error::new(format!("unexpected {}", tk.human()), self.source, span)
    }

    fn err_unexpected_character(&self, span: impl Into<Span>) -> Error {
        Error::new("unexpected character", self.source, span)
    }
}

impl Token {
    pub fn human(&self) -> &'static str {
        match self {
            Self::Raw => "raw template",
            Self::BeginExpr => "begin expression",
            Self::EndExpr => "end expression",
            Self::BeginBlock => "begin block",
            Self::EndBlock => "end block",
            Self::BeginComment => "begin comment",
            Self::EndComment => "end comment",
            Self::Period => "period",
            Self::Pipe => "pipe",
            Self::Comma => "comma",
            Self::Whitespace => "whitespace",
            Self::Ident => "identifier",
            Self::Keyword => "keyword",
        }
    }

    /// Returns the corresponding tag if this token is a tag.
    fn pair(&self) -> Self {
        match self {
            Self::BeginExpr => Self::EndExpr,
            Self::EndExpr => Self::BeginExpr,
            Self::BeginBlock => Self::EndBlock,
            Self::EndBlock => Self::BeginBlock,
            Self::BeginComment => Self::EndComment,
            Self::EndComment => Self::BeginComment,
            _ => panic!("not a tag"),
        }
    }

    fn is_begin_tag(&self) -> bool {
        matches!(
            self,
            Self::BeginExpr | Self::BeginBlock | Self::BeginComment
        )
    }

    fn is_begin_comment(&self) -> bool {
        matches!(self, Self::BeginComment)
    }

    fn is_whitespace(&self) -> bool {
        matches!(self, Self::Whitespace)
    }

    fn from_kind(tk: syntax::Kind) -> (Self, bool) {
        match tk {
            syntax::Kind::BeginExpr => (Token::BeginExpr, false),
            syntax::Kind::EndExpr => (Token::EndExpr, false),
            syntax::Kind::BeginExprTrim => (Token::BeginExpr, true),
            syntax::Kind::EndExprTrim => (Token::EndExpr, true),
            syntax::Kind::BeginBlock => (Token::BeginBlock, false),
            syntax::Kind::EndBlock => (Token::EndBlock, false),
            syntax::Kind::BeginBlockTrim => (Token::BeginBlock, true),
            syntax::Kind::EndBlockTrim => (Token::EndBlock, true),
            syntax::Kind::BeginComment => (Token::BeginComment, false),
            syntax::Kind::EndComment => (Token::EndComment, false),
            syntax::Kind::BeginCommentTrim => (Token::BeginComment, true),
            syntax::Kind::EndCommentTrim => (Token::EndComment, true),
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
    fn lex_begin_expr() {
        let tokens = lex("lorem ipsum {{").unwrap();
        assert_eq!(
            tokens,
            [(Token::Raw, "lorem ipsum "), (Token::BeginExpr, "{{"),]
        );
    }

    #[test]
    fn lex_begin_expr_trim() {
        let tokens = lex("lorem ipsum \t\n{{-").unwrap();
        assert_eq!(
            tokens,
            [(Token::Raw, "lorem ipsum"), (Token::BeginExpr, "{{-"),]
        );
    }

    #[test]
    fn lex_begin_expr_eof() {
        let tokens = lex("lorem ipsum {{ dolor").unwrap();
        assert_eq!(
            tokens,
            [
                (Token::Raw, "lorem ipsum "),
                (Token::BeginExpr, "{{"),
                (Token::Whitespace, " "),
                (Token::Ident, "dolor"),
            ]
        );
    }

    #[test]
    fn lex_end_expr_trim() {
        let tokens = lex("lorem ipsum {{ -}} \t\ndolor sit amet").unwrap();
        assert_eq!(
            tokens,
            [
                (Token::Raw, "lorem ipsum "),
                (Token::BeginExpr, "{{"),
                (Token::Whitespace, " "),
                (Token::EndExpr, "-}}"),
                (Token::Raw, "dolor sit amet")
            ]
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
    fn lex_expr_double_trim() {
        let tokens = lex("lorem {{ -}}  {{- }} dolor").unwrap();
        assert_eq!(
            tokens,
            [
                (Token::Raw, "lorem "),
                (Token::BeginExpr, "{{"),
                (Token::Whitespace, " "),
                (Token::EndExpr, "-}}"),
                (Token::Raw, ""),
                (Token::BeginExpr, "{{-"),
                (Token::Whitespace, " "),
                (Token::EndExpr, "}}"),
                (Token::Raw, " dolor")
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
    fn lex_expr_trim() {
        let tokens = lex("lorem ipsum    {{- .|\t aZ_0 -}}    dolor sit amet").unwrap();
        assert_eq!(
            tokens,
            [
                (Token::Raw, "lorem ipsum"),
                (Token::BeginExpr, "{{-"),
                (Token::Whitespace, " "),
                (Token::Period, "."),
                (Token::Pipe, "|"),
                (Token::Whitespace, "\t "),
                (Token::Ident, "aZ_0"),
                (Token::Whitespace, " "),
                (Token::EndExpr, "-}}"),
                (Token::Raw, "dolor sit amet"),
            ]
        );
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
    fn lex_block_and_expr() {
        let tokens =
            lex("{% if cond %} lorem ipsum {{ path.segment }} dolor sit amet {% end %}").unwrap();
        assert_eq!(
            tokens,
            [
                (Token::BeginBlock, "{%"),
                (Token::Whitespace, " "),
                (Token::Keyword, "if"),
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
    fn lex_begin_comment() {
        let tokens = lex("lorem ipsum {#").unwrap();
        assert_eq!(
            tokens,
            [(Token::Raw, "lorem ipsum "), (Token::BeginComment, "{#"),]
        );
    }

    #[test]
    fn lex_begin_comment_trim() {
        let tokens = lex("lorem ipsum \t\n{#-").unwrap();
        assert_eq!(
            tokens,
            [(Token::Raw, "lorem ipsum"), (Token::BeginComment, "{#-"),]
        );
    }

    #[test]
    fn lex_begin_comment_eof() {
        let tokens = lex("lorem ipsum {# dolor").unwrap();
        assert_eq!(
            tokens,
            [
                (Token::Raw, "lorem ipsum "),
                (Token::BeginComment, "{#"),
                (Token::Raw, " dolor")
            ]
        );
    }

    #[test]
    fn lex_end_comment_trim() {
        let tokens = lex("lorem ipsum {# -#} \t\ndolor sit amet").unwrap();
        assert_eq!(
            tokens,
            [
                (Token::Raw, "lorem ipsum "),
                (Token::BeginComment, "{#"),
                (Token::Raw, " "),
                (Token::EndComment, "-#}"),
                (Token::Raw, "dolor sit amet"),
            ]
        );
    }

    #[test]
    fn lex_empty_comment() {
        let tokens = lex("lorem ipsum {##}").unwrap();
        assert_eq!(
            tokens,
            [
                (Token::Raw, "lorem ipsum "),
                (Token::BeginComment, "{#"),
                (Token::EndComment, "#}"),
            ]
        );
    }

    #[test]
    fn lex_comment() {
        let tokens = lex("lorem ipsum {# anything goes e.g. - # { #}").unwrap();
        assert_eq!(
            tokens,
            [
                (Token::Raw, "lorem ipsum "),
                (Token::BeginComment, "{#"),
                (Token::Raw, " anything goes e.g. - # { "),
                (Token::EndComment, "#}"),
            ]
        );
    }

    #[test]
    fn lex_comment_trim() {
        let tokens = lex("lorem ipsum {# anything goes e.g. - # { #}").unwrap();
        assert_eq!(
            tokens,
            [
                (Token::Raw, "lorem ipsum "),
                (Token::BeginComment, "{#"),
                (Token::Raw, " anything goes e.g. - # { "),
                (Token::EndComment, "#}"),
            ]
        );
    }

    #[track_caller]
    fn lex(source: &str) -> Result<Vec<(Token, &str)>> {
        let engine = Engine::default();
        let mut lexer = Lexer::new(&engine, source);
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
