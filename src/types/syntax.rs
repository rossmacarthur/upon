use std::marker::PhantomData;

/// The template syntax configuration.
///
/// Use [`Syntax::default()`] to get the default syntax configuration and
/// [`Syntax::builder()`] to create a custom syntax configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Syntax<'a> {
    pub(crate) patterns: Vec<(Kind, String)>,
    _marker: PhantomData<&'a ()>,
}

/// A builder for the syntax configuration.
///
/// This struct is typically created using [`Syntax::builder()`].
#[derive(Debug, Clone)]
pub struct SyntaxBuilder<'a> {
    expr: Option<(&'a str, &'a str)>,
    block: Option<(&'a str, &'a str)>,
    comment: Option<(&'a str, &'a str)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    BeginExpr = 0,
    EndExpr = 1,
    BeginExprTrim = 2,
    EndExprTrim = 3,
    BeginBlock = 4,
    EndBlock = 5,
    BeginBlockTrim = 6,
    EndBlockTrim = 7,
    BeginComment = 8,
    EndComment = 9,
    BeginCommentTrim = 10,
    EndCommentTrim = 11,
}

#[test]
fn kind_usize() {
    for p in 0..12 {
        let k = Kind::from_usize(p);
        assert_eq!(k as usize, p);
    }
}

impl Default for Syntax<'_> {
    /// Returns the default syntax configuration.
    ///
    /// This is equivalent to the following.
    /// ```
    /// use upon::Syntax;
    ///
    /// let syntax = Syntax::builder()
    ///     .expr("{{", "}}")
    ///     .block("{%", "%}")
    ///     .comment("{#", "#}")
    ///     .build();
    /// assert_eq!(syntax, Syntax::default());
    /// ```
    #[inline]
    fn default() -> Self {
        Syntax::builder()
            .expr("{{", "}}")
            .block("{%", "%}")
            .comment("{#", "#}")
            .build()
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
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            expr: None,
            block: None,
            comment: None,
        }
    }

    /// Set the block syntax.
    ///
    /// If not set then the expression syntax will not be available.
    ///
    /// # Panics
    ///
    /// If either of the strings are empty.
    #[inline]
    pub fn expr(&mut self, begin_expr: &'a str, end_expr: &'a str) -> &mut Self {
        assert!(!begin_expr.is_empty() && !end_expr.is_empty());
        self.expr = Some((begin_expr, end_expr));
        self
    }

    /// Set the block syntax.
    ///
    /// If not set then the block syntax will not be available.
    ///
    /// # Panics
    ///
    /// If either of the strings are empty.
    #[inline]
    pub fn block(&mut self, begin_block: &'a str, end_block: &'a str) -> &mut Self {
        assert!(!begin_block.is_empty() && !end_block.is_empty());
        self.block = Some((begin_block, end_block));
        self
    }

    /// Set the comment syntax.
    ///
    /// If not set then comment syntax will not be available.
    ///
    /// # Panics
    ///
    /// If either of the strings are empty.
    #[inline]
    pub fn comment(&mut self, begin_comment: &'a str, end_comment: &'a str) -> &mut Self {
        assert!(!begin_comment.is_empty() && !end_comment.is_empty());
        self.comment = Some((begin_comment, end_comment));
        self
    }

    /// Builds the syntax configuration.
    pub fn build(&self) -> Syntax<'a> {
        let mut patterns = Vec::new();
        if let Some((begin, end)) = self.expr {
            patterns.push((Kind::BeginExpr, begin.into()));
            patterns.push((Kind::EndExpr, end.into()));
            patterns.push((Kind::BeginExprTrim, format!("{begin}-")));
            patterns.push((Kind::EndExprTrim, format!("-{end}")));
        };
        if let Some((begin, end)) = self.block {
            patterns.push((Kind::BeginBlock, begin.into()));
            patterns.push((Kind::EndBlock, end.into()));
            patterns.push((Kind::BeginBlockTrim, format!("{begin}-")));
            patterns.push((Kind::EndBlockTrim, format!("-{end}")));
        }
        if let Some((begin, end)) = self.comment {
            patterns.push((Kind::BeginComment, begin.into()));
            patterns.push((Kind::EndComment, end.into()));
            patterns.push((Kind::BeginCommentTrim, format!("{begin}-")));
            patterns.push((Kind::EndCommentTrim, format!("-{end}")));
        }
        Syntax {
            patterns,
            _marker: PhantomData,
        }
    }
}

impl Kind {
    pub fn from_usize(id: usize) -> Self {
        match id {
            0 => Self::BeginExpr,
            1 => Self::EndExpr,
            2 => Self::BeginExprTrim,
            3 => Self::EndExprTrim,
            4 => Self::BeginBlock,
            5 => Self::EndBlock,
            6 => Self::BeginBlockTrim,
            7 => Self::EndBlockTrim,
            8 => Self::BeginComment,
            9 => Self::EndComment,
            10 => Self::BeginCommentTrim,
            11 => Self::EndCommentTrim,
            _ => unreachable!(),
        }
    }
}

impl From<Kind> for usize {
    fn from(k: Kind) -> Self {
        k as usize
    }
}
