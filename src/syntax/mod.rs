mod ahocorasick;

use crate::syntax::ahocorasick::AhoCorasick;

#[derive(Debug)]
pub struct Searcher {
    imp: AhoCorasick,
}

/// The template syntax configuration.
///
/// Use [`Syntax::default()`] to get the default syntax configuration and
/// [`Syntax::builder()`] to create a custom syntax configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Syntax<'a> {
    patterns: Vec<(Kind, &'a str)>,
}

/// A builder for the syntax configuration.
///
/// This struct is typically created using [`Syntax::builder()`].
#[derive(Debug, Clone)]
pub struct SyntaxBuilder<'a> {
    expr: Option<(&'a str, &'a str)>,
    block: Option<(&'a str, &'a str)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    /// Begin expression delimiter, e.g. `{{`
    BeginExpr = 0,
    /// End expression delimiter, e.g. `}}`
    EndExpr = 1,
    /// Begin block delimiter, e.g. `{%`
    BeginBlock = 2,
    /// End block delimiter, e.g. `%}`
    EndBlock = 3,
}

impl Searcher {
    pub fn new(syntax: Syntax) -> Self {
        let imp = AhoCorasick::new(syntax.patterns.into_iter().map(|(k, v)| (k as usize, v)));
        Self { imp }
    }

    pub fn find_at<T>(&self, haystack: T, at: usize) -> Option<(Kind, usize, usize)>
    where
        T: AsRef<[u8]>,
    {
        self.imp.find_at(haystack, at).map(|m| {
            let kind = Kind::from_usize(m.pattern_id());
            (kind, m.start(), m.end())
        })
    }

    pub fn starts_with<T>(&self, haystack: T, at: usize) -> Option<(Kind, usize)>
    where
        T: AsRef<[u8]>,
    {
        let (kind, i, j) = self.find_at(haystack, at)?;
        (at == i).then(|| (kind, j))
    }
}

impl Default for Syntax<'_> {
    /// Returns the default syntax configuration.
    ///
    /// This is equivalent to the following.
    /// ```
    /// use upon::Syntax;
    ///
    /// let syntax = Syntax::builder().expr("{{", "}}").block("{%", "%}").build();
    /// assert_eq!(syntax, Syntax::default());
    /// ```
    #[inline]
    fn default() -> Self {
        Syntax::builder().expr("{{", "}}").block("{%", "%}").build()
    }
}

impl<'a> Syntax<'a> {
    /// Create a new syntax builder.
    ///
    /// # Examples
    ///
    /// ```
    /// let syntax = upon::Syntax::builder()
    ///     .expr("<{", "}>")
    ///     .block("<[", "]>")
    ///     .build();
    /// ```
    #[inline]
    pub fn builder() -> SyntaxBuilder<'a> {
        SyntaxBuilder::new()
    }
}

impl<'a> SyntaxBuilder<'a> {
    /// Creates a new syntax builder.
    #[inline]
    pub fn new() -> Self {
        Self {
            expr: None,
            block: None,
        }
    }

    /// Set the block syntax.
    ///
    /// If not set then the expression syntax will not be available.
    #[inline]
    pub fn expr(&mut self, begin_expr: &'a str, end_expr: &'a str) -> &mut Self {
        self.expr = Some((begin_expr, end_expr));
        self
    }

    /// Set the block syntax.
    ///
    /// If not set then block syntax will not be available.
    #[inline]
    pub fn block(&mut self, begin_block: &'a str, end_block: &'a str) -> &mut Self {
        self.block = Some((begin_block, end_block));
        self
    }

    /// Builds the syntax configuration.
    pub fn build(&self) -> Syntax<'a> {
        let mut patterns = Vec::new();
        if let Some((begin, end)) = self.expr {
            patterns.push((Kind::BeginExpr, begin));
            patterns.push((Kind::EndExpr, end));
        };
        if let Some((begin, end)) = self.block {
            patterns.push((Kind::BeginBlock, begin));
            patterns.push((Kind::EndBlock, end));
        }
        Syntax { patterns }
    }
}

impl Kind {
    fn from_usize(id: usize) -> Self {
        match id {
            0 => Kind::BeginExpr,
            1 => Kind::EndExpr,
            2 => Kind::BeginBlock,
            3 => Kind::EndBlock,
            _ => unreachable!(),
        }
    }
}
