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
//! should defined functions or closures that adhere to the generic
//! implementation provided.
//!
//! ## Argument types
//!
//! [`Filter`] is implemented for functions and closures that take any of the
//! following owned types as arguments. See [`FilterArg`].
//! - [`bool`]
//! - [`i64`]
//! - [`f64`]
//! - [`String`]
//! - [`Vec<Value>`]
//! - [`BTreeMap<String, Value>`][std::collections::BTreeMap]
//! - [`Value`]
//!
//! **Additionally:**
//! - The _first_ argument to the filter (i.e. the piped expression) can also be
//!   specified using the following reference types. This is preferred in most
//!   cases because the renderer won't have to clone the value before the
//!   calling the filter.
//!   - [`&str`][str]
//!   - [`&[Value]`][slice]
//!   - [`&BTreeMap<String, Value>`][std::collections::BTreeMap]
//!   - [`&Value`][Value]
//!
//! - Other arguments can also use [`&str`][str] but only if the passed
//!   parameter is a literal string.
//!
//! ## Return types
//!
//! [`Filter`] is implemented for functions and closures that return any of the
//! following types. See [`FilterReturn`].
//!
//! - `R` where `R` implements `Into<Value>`
//! - `Option<R>` where `R` implements `Into<Value>`
//! - `Result<R>` where `R` implements `Into<Value>`
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

use crate::render::{FilterState, Stack};
use crate::types::ast::BaseExpr;
use crate::types::span::Span;
use crate::value::ValueCow;
use crate::{Error, Result, Value};

pub(crate) type FilterFn = dyn Fn(FilterState<'_>) -> Result<Value> + Send + Sync + 'static;

pub(crate) fn new<F, R, A>(f: F) -> Box<FilterFn>
where
    F: Filter<R, A> + Send + Sync + 'static,
    R: FilterReturn,
    A: for<'a> FilterArgs<'a>,
{
    Box::new(move |state: FilterState<'_>| -> Result<Value> {
        let args = A::from_state(state)?;
        let result = Filter::filter(&f, args);
        FilterReturn::to_value(result)
    })
}

/// Represents any filter function.
///
/// See the [module][crate::filters] documentation for more information.
#[cfg_attr(docsrs, doc(cfg(feature = "filters")))]
pub trait Filter<R, A>
where
    A: for<'a> FilterArgs<'a>,
{
    #[doc(hidden)]
    fn filter(&self, args: <A as FilterArgs<'_>>::Output) -> R;
}

/// Represents *all* the arguments to a filter.
///
/// See the [module][crate::filters] documentation for more information.
#[cfg_attr(docsrs, doc(cfg(feature = "filters")))]
pub trait FilterArgs<'a> {
    #[doc(hidden)]
    type Output: 'a;
    #[doc(hidden)]
    fn from_state(state: FilterState<'a>) -> Result<Self::Output>;
}

/// Represents an argument to a filter.
///
/// See the [module][crate::filters] documentation for more information.
#[cfg_attr(docsrs, doc(cfg(feature = "filters")))]
pub trait FilterArg<'a> {
    #[doc(hidden)]
    type Output: 'a;
    #[doc(hidden)]
    fn from_value(v: Value) -> args::Result<Self::Output>;
    #[doc(hidden)]
    fn from_value_ref(v: &'a Value) -> args::Result<Self::Output>;
    #[doc(hidden)]
    fn from_cow_mut(v: &'a mut ValueCow<'a>) -> args::Result<Self::Output>;
}

/// Represents a return value from a filter.
///
/// See the [module][crate::filters] documentation for more information.
#[cfg_attr(docsrs, doc(cfg(feature = "filters")))]
pub trait FilterReturn {
    #[doc(hidden)]
    fn to_value(self) -> Result<Value>;
}

////////////////////////////////////////////////////////////////////////////////
// Filter
////////////////////////////////////////////////////////////////////////////////

impl<Func, R, V> Filter<R, (V,)> for Func
where
    Func: Fn(V) -> R,
    R: FilterReturn,

    V: for<'a> FilterArg<'a, Output = V>,

    (V,): for<'a> FilterArgs<'a, Output = (V,)>,
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

    V: for<'a> FilterArg<'a, Output = V>,
    A: for<'a> FilterArg<'a, Output = A>,

    (V, A): for<'a> FilterArgs<'a, Output = (V, A)>,
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

    V: for<'a> FilterArg<'a, Output = V>,
    A: for<'a> FilterArg<'a, Output = A>,
    B: for<'a> FilterArg<'a, Output = B>,

    (V, A, B): for<'a> FilterArgs<'a, Output = (V, A, B)>,
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

    V: for<'a> FilterArg<'a, Output = V>,
    A: for<'a> FilterArg<'a, Output = A>,
    B: for<'a> FilterArg<'a, Output = B>,
    C: for<'a> FilterArg<'a, Output = C>,

    (V, A, B, C): for<'a> FilterArgs<'a, Output = (V, A, B, C)>,
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

    V: for<'a> FilterArg<'a, Output = V>,
    A: for<'a> FilterArg<'a, Output = A>,
    B: for<'a> FilterArg<'a, Output = B>,
    C: for<'a> FilterArg<'a, Output = C>,
    D: for<'a> FilterArg<'a, Output = D>,

    (V, A, B, C, D): for<'a> FilterArgs<'a, Output = (V, A, B, C, D)>,
{
    #[doc(hidden)]
    fn filter<'a>(&self, (v, a, b, c, d): (V, A, B, C, D)) -> R {
        self(v, a, b, c, d)
    }
}

////////////////////////////////////////////////////////////////////////////////
// FilterArgs
////////////////////////////////////////////////////////////////////////////////

impl<'a, V> FilterArgs<'a> for (V,)
where
    V: FilterArg<'a>,
{
    type Output = (V::Output,);

    fn from_state(state: FilterState<'a>) -> Result<Self::Output> {
        check_args(&state, 0)?;
        let err = |e| err_expected_val(e, state.source, state.filter.span);
        let v = V::from_cow_mut(state.value).map_err(err)?;
        Ok((v,))
    }
}
impl<'a, V, A> FilterArgs<'a> for (V, A)
where
    V: FilterArg<'a>,
    A: FilterArg<'a>,
{
    type Output = (V::Output, A::Output);

    fn from_state(state: FilterState<'a>) -> Result<Self::Output> {
        check_args(&state, 1)?;
        let err = |e| err_expected_val(e, state.source, state.filter.span);
        let v = V::from_cow_mut(state.value).map_err(err)?;
        let a = get_arg::<A>(state.source, state.stack, state.args, 0)?;
        Ok((v, a))
    }
}

impl<'a, V, A, B> FilterArgs<'a> for (V, A, B)
where
    V: FilterArg<'a>,
    A: FilterArg<'a>,
    B: FilterArg<'a>,
{
    type Output = (V::Output, A::Output, B::Output);

    fn from_state(state: FilterState<'a>) -> Result<Self::Output> {
        check_args(&state, 2)?;
        let err = |e| err_expected_val(e, state.source, state.filter.span);
        let v = V::from_cow_mut(state.value).map_err(err)?;
        let a = get_arg::<A>(state.source, state.stack, state.args, 0)?;
        let b = get_arg::<B>(state.source, state.stack, state.args, 1)?;
        Ok((v, a, b))
    }
}

impl<'a, V, A, B, C> FilterArgs<'a> for (V, A, B, C)
where
    V: FilterArg<'a>,
    A: FilterArg<'a>,
    B: FilterArg<'a>,
    C: FilterArg<'a>,
{
    type Output = (V::Output, A::Output, B::Output, C::Output);

    fn from_state(state: FilterState<'a>) -> Result<Self::Output> {
        check_args(&state, 3)?;
        let err = |e| err_expected_val(e, state.source, state.filter.span);
        let v = V::from_cow_mut(state.value).map_err(err)?;
        let a = get_arg::<A>(state.source, state.stack, state.args, 0)?;
        let b = get_arg::<B>(state.source, state.stack, state.args, 1)?;
        let c = get_arg::<C>(state.source, state.stack, state.args, 2)?;
        Ok((v, a, b, c))
    }
}

impl<'a, V, A, B, C, D> FilterArgs<'a> for (V, A, B, C, D)
where
    V: FilterArg<'a>,
    A: FilterArg<'a>,
    B: FilterArg<'a>,
    C: FilterArg<'a>,
    D: FilterArg<'a>,
{
    type Output = (V::Output, A::Output, B::Output, C::Output, D::Output);

    fn from_state(state: FilterState<'a>) -> Result<Self::Output> {
        check_args(&state, 4)?;
        let err = |e| err_expected_val(e, state.source, state.filter.span);
        let v = V::from_cow_mut(state.value).map_err(err)?;
        let a = get_arg::<A>(state.source, state.stack, state.args, 0)?;
        let b = get_arg::<B>(state.source, state.stack, state.args, 1)?;
        let c = get_arg::<C>(state.source, state.stack, state.args, 2)?;
        let d = get_arg::<D>(state.source, state.stack, state.args, 3)?;
        Ok((v, a, b, c, d))
    }
}

fn check_args(state: &FilterState<'_>, exp: usize) -> Result<()> {
    if state.args.len() == exp {
        Ok(())
    } else {
        Err(Error::render(
            format!("filter expected {} arguments", exp),
            state.source,
            state.filter.span,
        ))
    }
}

fn get_arg<'a, T>(
    source: &str,
    stack: &'a Stack<'a>,
    args: &'a [BaseExpr],
    i: usize,
) -> Result<T::Output>
where
    T: FilterArg<'a>,
{
    match &args[i] {
        BaseExpr::Var(var) => match stack.lookup_path(source, &var.path)? {
            ValueCow::Borrowed(v) => {
                T::from_value_ref(v).map_err(|e| err_expected_arg(e, source, var.span))
            }
            ValueCow::Owned(v) => {
                T::from_value(v).map_err(|e| err_expected_arg(e, source, var.span))
            }
        },
        BaseExpr::Literal(lit) => {
            T::from_value_ref(&lit.value).map_err(|e| err_expected_arg(e, source, lit.span))
        }
    }
}

fn err_expected_arg(err: args::Error, source: &str, span: Span) -> Error {
    let msg = match err {
        args::Error::Type(exp, got) => {
            format!("filter expected {} argument, found {}", exp, got)
        }
        args::Error::Reference(got) => {
            format!(
                "filter expected reference argument but this {} can only be passed as owned",
                got
            )
        }
    };
    Error::render(msg, source, span)
}

fn err_expected_val(err: args::Error, source: &str, span: Span) -> Error {
    let msg = match err {
        args::Error::Type(exp, got) => {
            format!("filter expected {} value, found {}", exp, got)
        }
        args::Error::Reference(_) => {
            unreachable!()
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

impl<T> FilterReturn for Option<T>
where
    T: Into<Value>,
{
    fn to_value(self) -> Result<Value> {
        match self {
            Some(r) => Ok(r.into()),
            None => Ok(Value::None),
        }
    }
}

impl<T> FilterReturn for Result<T>
where
    T: Into<Value>,
{
    fn to_value(self) -> Result<Value> {
        self.map(Into::into)
    }
}
