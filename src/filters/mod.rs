//! An abstraction over any filter function or closure.
//!
//! The [`Filter`] trait is used by the
//! [`Engine::add_filter`][crate::Engine::add_filter] method to abstract over a
//! variety of function and closure types. This includes filters with variable
//! argument types, return types and arity. The first argument to a filter
//! function will always receive the piped value or expression. It can then have
//! up to four more arguments. The renderer will check the number of arguments
//! and the type of arguments when the filter is used. Generally you should not
//! try to implement any of the traits in this module yourself, instead you
//! should define functions or closures that adhere to the generic
//! implementation provided.
//!
//! ## Types
//!
//! [`Filter`] is implemented for functions and closures that take any owned
//! argument implementing [`FilterArg`] and any return type implementing
//! [`FilterReturn`].
//!
//! Additionally, the _first_ argument to the filter (i.e. the piped expression)
//! can also be specified using the following reference types. This is preferred
//! in most cases because the renderer won't have to clone the value before
//! passing it to the filter.
//! - [`&str`][str]
//! - [`&[Value]`][slice]
//! - [`&BTreeMap<String, Value>`][std::collections::BTreeMap]
//! - [`&Value`][Value]
//!
//! Other arguments can also use [`&str`][str] but only if the passed parameter
//! is always a literal string.
//!
//! # Examples
//!
//! ## Using existing functions
//!
//! A lot of standard library functions can be used as filters, as long as they
//! have the supported argument and return types.
//!
//! ```
//! let mut engine = upon::Engine::new();
//! engine.add_filter("lower", str::to_lowercase);
//! engine.add_filter("abs", i64::abs);
//! ```
//!
//! ## Closures
//!
//! Closures are perfectly valid filters.
//!
//! ```
//! let mut engine = upon::Engine::new();
//! engine.add_filter("add", |a: i64, b: i64| a + b);
//! ```
//!
//! This could be use like this
//!
//! ```text
//! {{ user.age | add: 10 }}
//! ```
//!
//! ## Owned vs reference arguments
//!
//! Consider the following template.
//!
//! ```text
//! {{ users | last }}
//! ```
//!
//! Where the `last` filter retrieves the final element in a list. We could
//! implement this filter taking an owned argument.
//!
//! ```rust
//! # use upon::Value;
//! fn last(mut list: Vec<Value>) -> Option<Value> {
//!     list.pop()
//! }
//! ```
//!
//! But it would be more efficient to implement it such that it takes a slice,
//! because then only the last element is cloned, as opposed to all the elements
//! in the list being cloned.
//!
//! ```
//! # use upon::Value;
//! fn last(list: &[Value]) -> Option<Value> {
//!     list.last().map(Clone::clone)
//! }
//! ```

mod args;
mod impls;

use crate::render::FilterState;
use crate::types::span::Span;
use crate::value::ValueCow;
use crate::{Error, Result, Value};

pub(crate) type FilterFn = dyn Fn(FilterState<'_, '_>) -> Result<Value> + Send + Sync + 'static;

pub(crate) fn new<F, R, A>(f: F) -> Box<FilterFn>
where
    F: Filter<R, A> + Send + Sync + 'static,
    R: FilterReturn,
    A: FilterArgs,
{
    Box::new(move |state: FilterState<'_, '_>| -> Result<Value> {
        let args = A::from_state(state)?;
        let result = Filter::filter(&f, args);
        FilterReturn::to_value(result)
    })
}

/// Any filter function.
///
/// *See the [module][crate::filters] documentation for more information.*
#[cfg_attr(docsrs, doc(cfg(feature = "filters")))]
pub trait Filter<R, A>
where
    A: FilterArgs,
{
    #[doc(hidden)]
    fn filter(&self, args: <A as FilterArgs>::Output<'_>) -> R;
}

/// The set of arguments to a filter.
///
/// *See the [module][crate::filters] documentation for more information.*
#[cfg_attr(docsrs, doc(cfg(feature = "filters")))]
pub trait FilterArgs {
    #[doc(hidden)]
    type Output<'args>;
    #[doc(hidden)]
    fn from_state<'args>(state: FilterState<'_, 'args>) -> Result<Self::Output<'args>>;
}

/// An argument to a filter.
///
/// *See the [module][crate::filters] documentation for more information.*
#[cfg_attr(docsrs, doc(cfg(feature = "filters")))]
pub trait FilterArg {
    #[doc(hidden)]
    type Output<'arg>;
    #[doc(hidden)]
    fn from_value<'stack, 'arg>(v: &'arg mut ValueCow<'stack>) -> args::Result<Self::Output<'arg>>
    where
        'stack: 'arg;
}

/// A return value from a filter.
///
/// This trait is implemented for many types by utilizing the [`From`]
/// implementations for [`Value`].
///
/// - `R` where `R` implements `Into<Value>`
/// - `Result<R, E>` where `R` implements `Into<Value>` and `E` implements
///   [`FilterError`].
///
/// *See the [module][crate::filters] documentation for more information.*
#[cfg_attr(docsrs, doc(cfg(feature = "filters")))]
pub trait FilterReturn {
    #[doc(hidden)]
    fn to_value(self) -> Result<Value>;
}

/// A value returned from a filter.
///
/// *See the [module][crate::filters] documentation for more information.*
pub trait FilterError {
    #[doc(hidden)]
    fn to_error(self) -> Error;
}

////////////////////////////////////////////////////////////////////////////////
// Filter
////////////////////////////////////////////////////////////////////////////////

impl<Func, R, V> Filter<R, (V,)> for Func
where
    Func: Fn(V) -> R,
    R: FilterReturn,

    V: for<'a> FilterArg<Output<'a> = V>,

    (V,): for<'a> FilterArgs<Output<'a> = (V,)>,
{
    #[doc(hidden)]
    fn filter<'a>(&self, (v,): (V,)) -> R {
        self(v)
    }
}

impl<Func, R, V, A> Filter<R, (V, A)> for Func
where
    Func: Fn(V, A) -> R,
    R: FilterReturn,

    V: for<'a> FilterArg<Output<'a> = V>,
    A: for<'a> FilterArg<Output<'a> = A>,

    (V, A): for<'a> FilterArgs<Output<'a> = (V, A)>,
{
    #[doc(hidden)]
    fn filter<'a>(&self, (v, a): (V, A)) -> R {
        self(v, a)
    }
}

impl<Func, R, V, A, B> Filter<R, (V, A, B)> for Func
where
    Func: Fn(V, A, B) -> R,
    R: FilterReturn,

    V: for<'a> FilterArg<Output<'a> = V>,
    A: for<'a> FilterArg<Output<'a> = A>,
    B: for<'a> FilterArg<Output<'a> = B>,

    (V, A, B): for<'a> FilterArgs<Output<'a> = (V, A, B)>,
{
    #[doc(hidden)]
    fn filter<'a>(&self, (v, a, b): (V, A, B)) -> R {
        self(v, a, b)
    }
}

impl<Func, R, V, A, B, C> Filter<R, (V, A, B, C)> for Func
where
    Func: Fn(V, A, B, C) -> R,
    R: FilterReturn,

    V: for<'a> FilterArg<Output<'a> = V>,
    A: for<'a> FilterArg<Output<'a> = A>,
    B: for<'a> FilterArg<Output<'a> = B>,
    C: for<'a> FilterArg<Output<'a> = C>,

    (V, A, B, C): for<'a> FilterArgs<Output<'a> = (V, A, B, C)>,
{
    #[doc(hidden)]
    fn filter<'a>(&self, (v, a, b, c): (V, A, B, C)) -> R {
        self(v, a, b, c)
    }
}

impl<Func, R, V, A, B, C, D> Filter<R, (V, A, B, C, D)> for Func
where
    Func: Fn(V, A, B, C, D) -> R,
    R: FilterReturn,

    V: for<'a> FilterArg<Output<'a> = V>,
    A: for<'a> FilterArg<Output<'a> = A>,
    B: for<'a> FilterArg<Output<'a> = B>,
    C: for<'a> FilterArg<Output<'a> = C>,
    D: for<'a> FilterArg<Output<'a> = D>,

    (V, A, B, C, D): for<'a> FilterArgs<Output<'a> = (V, A, B, C, D)>,
{
    #[doc(hidden)]
    fn filter<'a>(&self, (v, a, b, c, d): (V, A, B, C, D)) -> R {
        self(v, a, b, c, d)
    }
}

////////////////////////////////////////////////////////////////////////////////
// FilterArgs
////////////////////////////////////////////////////////////////////////////////

impl<V> FilterArgs for (V,)
where
    V: FilterArg,
{
    type Output<'a> = (V::Output<'a>,);

    fn from_state<'args>(state: FilterState<'_, 'args>) -> Result<Self::Output<'args>> {
        let [(v, _)] = get_args(state.args)?;
        let v = V::from_value(v).map_err(err_expected_val)?;
        Ok((v,))
    }
}

impl<V, A> FilterArgs for (V, A)
where
    V: FilterArg,
    A: FilterArg,
{
    type Output<'a> = (V::Output<'a>, A::Output<'a>);

    fn from_state<'args>(state: FilterState<'_, 'args>) -> Result<Self::Output<'args>> {
        let err = |e, sp| err_expected_arg(e, state.source, sp);
        let [(v, _), (a, sa)] = get_args(state.args)?;
        let v = V::from_value(v).map_err(err_expected_val)?;
        let a = A::from_value(a).map_err(|e| err(e, *sa))?;
        Ok((v, a))
    }
}

impl<V, A, B> FilterArgs for (V, A, B)
where
    V: FilterArg,
    A: FilterArg,
    B: FilterArg,
{
    type Output<'a> = (V::Output<'a>, A::Output<'a>, B::Output<'a>);

    fn from_state<'args>(state: FilterState<'_, 'args>) -> Result<Self::Output<'args>> {
        let err = |e, sp| err_expected_arg(e, state.source, sp);
        let [(v, _), (a, sa), (b, sb)] = get_args(state.args)?;
        let v = V::from_value(v).map_err(err_expected_val)?;
        let a = A::from_value(a).map_err(|e| err(e, *sa))?;
        let b = B::from_value(b).map_err(|e| err(e, *sb))?;
        Ok((v, a, b))
    }
}

impl<V, A, B, C> FilterArgs for (V, A, B, C)
where
    V: FilterArg,
    A: FilterArg,
    B: FilterArg,
    C: FilterArg,
{
    type Output<'a> = (V::Output<'a>, A::Output<'a>, B::Output<'a>, C::Output<'a>);

    fn from_state<'args>(state: FilterState<'_, 'args>) -> Result<Self::Output<'args>> {
        let err = |e, sp| err_expected_arg(e, state.source, sp);
        let [(v, _), (a, sa), (b, sb), (c, sc)] = get_args(state.args)?;
        let v = V::from_value(v).map_err(err_expected_val)?;
        let a = A::from_value(a).map_err(|e| err(e, *sa))?;
        let b = B::from_value(b).map_err(|e| err(e, *sb))?;
        let c = C::from_value(c).map_err(|e| err(e, *sc))?;
        Ok((v, a, b, c))
    }
}

impl<V, A, B, C, D> FilterArgs for (V, A, B, C, D)
where
    V: FilterArg,
    A: FilterArg,
    B: FilterArg,
    C: FilterArg,
    D: FilterArg,
{
    type Output<'a> = (
        V::Output<'a>,
        A::Output<'a>,
        B::Output<'a>,
        C::Output<'a>,
        D::Output<'a>,
    );

    fn from_state<'args>(state: FilterState<'_, 'args>) -> Result<Self::Output<'args>> {
        let err = |e, sp| err_expected_arg(e, state.source, sp);
        let [(v, _), (a, sa), (b, sb), (c, sc), (d, sd)] = get_args(state.args)?;
        let v = V::from_value(v).map_err(err_expected_val)?;
        let a = A::from_value(a).map_err(|e| err(e, *sa))?;
        let b = B::from_value(b).map_err(|e| err(e, *sb))?;
        let c = C::from_value(c).map_err(|e| err(e, *sc))?;
        let d = D::from_value(d).map_err(|e| err(e, *sd))?;
        Ok((v, a, b, c, d))
    }
}

fn get_args<'stack, 'args, const N: usize>(
    args: &'args mut [(ValueCow<'stack>, Span)],
) -> Result<&'args mut [(ValueCow<'stack>, Span); N]> {
    let n = args.len() - 1;
    args.try_into().map_err(|_| {
        Error::render_plain(format!(
            "filter expects {} arguments, {} provided",
            N - 1,
            n
        ))
    })
}

fn err_expected_val(err: args::Error) -> Error {
    let msg = match err {
        args::Error::Type(exp, got) => {
            format!("filter expects {exp} value, found {got}")
        }
        args::Error::TryFromInt(want, value) => {
            format!("filter expects {want} value, but `{value}` is out of range",)
        }
    };
    Error::render_plain(msg)
}
fn err_expected_arg(err: args::Error, source: &str, span: Span) -> Error {
    let msg = match err {
        args::Error::Type(exp, got) => {
            format!("filter expects {exp} argument, found {got}")
        }
        args::Error::TryFromInt(want, value) => {
            format!("filter expects {want} argument, but `{value}` is out of range",)
        }
    };
    Error::render(msg, source, span)
}

////////////////////////////////////////////////////////////////////////////////
// FilterReturn
////////////////////////////////////////////////////////////////////////////////

impl<T> FilterReturn for T
where
    T: Into<Value>,
{
    fn to_value(self) -> Result<Value> {
        Ok(self.into())
    }
}

impl<T, E> FilterReturn for std::result::Result<T, E>
where
    T: Into<Value>,
    E: FilterError,
{
    fn to_value(self) -> Result<Value> {
        self.map(Into::into).map_err(FilterError::to_error)
    }
}

////////////////////////////////////////////////////////////////////////////////
// FilterError
////////////////////////////////////////////////////////////////////////////////

impl FilterError for String {
    fn to_error(self) -> Error {
        Error::filter(self)
    }
}

impl FilterError for &str {
    fn to_error(self) -> Error {
        Error::filter(self)
    }
}
