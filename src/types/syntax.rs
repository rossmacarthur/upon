use std::marker::PhantomData;

use crate::types::delimiter::Delimiter;

/// The template syntax configuration.
///
/// Use [`Syntax::default()`] to get the default syntax configuration and
/// [`Syntax::builder()`] to create a custom syntax configuration.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(internal_debug, derive(Debug))]
pub struct Syntax<'a> {
    /// The corresponding delimiters for the patterns.
    pub(crate) delimiters: Vec<Delimiter>,
    /// The configured patterns.
    pub(crate) patterns: Vec<String>,
    _marker: PhantomData<&'a ()>,
}

/// A builder for the syntax configuration.
///
/// This struct is created using [`Syntax::builder()`].
#[derive(Debug, Clone)]
pub struct SyntaxBuilder<'a> {
    expr: Option<(&'a str, &'a str)>,
    block: Option<(&'a str, &'a str)>,
    comment: Option<(&'a str, &'a str)>,
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

#[cfg(not(internal_debug))]
impl std::fmt::Debug for Syntax<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Syntax").finish_non_exhaustive()
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
        SyntaxBuilder {
            expr: None,
            block: None,
            comment: None,
        }
    }
}

impl<'a> SyntaxBuilder<'a> {
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
        let mut delimiters = Vec::new();
        let mut patterns = Vec::new();
        let mut push = |delimiter, pattern| {
            delimiters.push(delimiter);
            patterns.push(pattern);
        };
        if let Some((begin, end)) = self.expr {
            push(Delimiter::BeginExpr, begin.into());
            push(Delimiter::EndExpr, end.into());
            push(Delimiter::BeginExprTrim, format!("{begin}-"));
            push(Delimiter::EndExprTrim, format!("-{end}"));
        };
        if let Some((begin, end)) = self.block {
            push(Delimiter::BeginBlock, begin.into());
            push(Delimiter::EndBlock, end.into());
            push(Delimiter::BeginBlockTrim, format!("{begin}-"));
            push(Delimiter::EndBlockTrim, format!("-{end}"));
        }
        if let Some((begin, end)) = self.comment {
            push(Delimiter::BeginComment, begin.into());
            push(Delimiter::EndComment, end.into());
            push(Delimiter::BeginCommentTrim, format!("{begin}-"));
            push(Delimiter::EndCommentTrim, format!("-{end}"));
        }
        Syntax {
            delimiters,
            patterns,
            _marker: PhantomData,
        }
    }
}
