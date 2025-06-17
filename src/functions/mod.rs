//! An abstraction over any function or closure.
//!
//! The [`Function`] trait is used by the
//! [`Engine::add_function`][crate::Engine::add_function] method to abstract
//! over a variety of function and closure types. This includes functions with
//! variable argument types, return types and arity. When used as a *filter* the
//! first argument to the function will always receive the piped value or
//! expression. It can then have up to four more arguments. The renderer will
//! check the number of arguments and the type of arguments when the function is
//! used. Generally you should not try to implement any of the traits in this
//! module yourself, instead you should define functions or closures that adhere
//! to the generic implementation provided.
//!
//! ## Types
//!
//! [`Function`] is implemented for functions and closures that take any owned
//! argument implementing [`FunctionArg`] and any return type implementing
//! [`FunctionReturn`].
//!
//! The _first_ argument to the function (i.e. the piped expression when used as
//! a *filter*) can also be specified using the following reference types. This
//! is preferred in most cases because the renderer won't have to clone the
//! value before passing it to the function.
//! - [`&str`][str]
//! - [`&[Value]`][slice]
//! - [`&BTreeMap<String, Value>`][std::collections::BTreeMap]
//! - [`&Value`][Value]
//!
//! The _second_ argument can be specified using the following reference types.
//! - [`&str`][str]
//! - [`&Value`][Value]
//!
//! Other arguments can always use [`&str`][str].
//!
//! The technical reason for this contraint is that each permutation of
//! _reference_ arguments needs its own trait implementation.
//!
//! # Examples
//!
//! ## Using existing functions
//!
//! A lot of standard library functions and existing functions satisfy the
//! [`Function`] trait, as long as they have the supported argument and return
//! types.
//!
//! ```
//! let mut engine = upon::Engine::new();
//! engine.add_function("lower", str::to_lowercase);
//! engine.add_function("abs", i64::abs);
//! engine.add_function("eq", upon::Value::eq);
//! ```
//!
//! ## Closures
//!
//! Closures are perfectly valid functions, although often they will need type
//! hints for the arguments.
//!
//! ```
//! let mut engine = upon::Engine::new();
//! engine.add_function("add", |a: i64, b: i64| a + b);
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
//! Where the `last` function retrieves the final element in a list. We could
//! implement this function taking an owned argument.
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
//!     list.last().cloned()
//! }
//! ```

mod args;
mod impls;

use crate::render::FunctionState;
use crate::types::span::Span;
use crate::value::ValueCow;
use crate::{Error, Result, Value};

pub(crate) type DynFunction =
    dyn Fn(FunctionState<'_, '_>) -> Result<Value> + Send + Sync + 'static;

pub(crate) fn new<F, R, A>(f: F) -> Box<DynFunction>
where
    F: Function<R, A> + Send + Sync + 'static,
    R: FunctionReturn,
    A: FunctionArgs,
{
    Box::new(move |state: FunctionState<'_, '_>| -> Result<Value> {
        let args = A::from_state(state)?;
        let result = Function::call(&f, args);
        FunctionReturn::to_value(result)
    })
}

/// Any function.
///
/// Some implementations of this trait are hidden.
///
/// *See the [module][crate::functions] documentation for more information.*
#[cfg_attr(docsrs, doc(cfg(feature = "functions")))]
pub trait Function<R, A>
where
    A: FunctionArgs,
{
    #[doc(hidden)]
    fn call(&self, args: <A as FunctionArgs>::Output<'_>) -> R;
}

/// The set of arguments to a function.
///
/// *See the [module][crate::functions] documentation for more information.*
#[cfg_attr(docsrs, doc(cfg(feature = "functions")))]
pub trait FunctionArgs {
    #[doc(hidden)]
    type Output<'args>;
    #[doc(hidden)]
    fn from_state<'args>(state: FunctionState<'_, 'args>) -> Result<Self::Output<'args>>;
}

/// An argument to a function.
///
/// *See the [module][crate::functions] documentation for more information.*
#[cfg_attr(docsrs, doc(cfg(feature = "functions")))]
pub trait FunctionArg {
    #[doc(hidden)]
    type Output<'arg>;
    #[doc(hidden)]
    fn from_value<'stack, 'arg>(v: &'arg mut ValueCow<'stack>) -> args::Result<Self::Output<'arg>>
    where
        'stack: 'arg;
}

/// A return value from a function.
///
/// This trait is implemented for many types by utilizing the [`From`]
/// implementations for [`Value`].
///
/// - `R` where `R` implements `Into<Value>`
/// - `Result<R, E>` where `R` implements `Into<Value>` and `E` implements
///   [`FunctionError`].
///
/// *See the [module][crate::functions] documentation for more information.*
#[cfg_attr(docsrs, doc(cfg(feature = "functions")))]
pub trait FunctionReturn {
    #[doc(hidden)]
    fn to_value(self) -> Result<Value>;
}

/// An error returned from a function.
///
/// *See the [module][crate::functions] documentation for more information.*
pub trait FunctionError {
    #[doc(hidden)]
    fn to_error(self) -> Error;
}

////////////////////////////////////////////////////////////////////////////////
// Function
////////////////////////////////////////////////////////////////////////////////

impl<Func, R> Function<R, ()> for Func
where
    Func: Fn() -> R,
    R: FunctionReturn,
{
    #[doc(hidden)]
    fn call<'a>(&self, (): ()) -> R {
        self()
    }
}

impl<Func, R, A> Function<R, (A,)> for Func
where
    Func: Fn(A) -> R,
    R: FunctionReturn,

    A: for<'a> FunctionArg<Output<'a> = A>,

    (A,): for<'a> FunctionArgs<Output<'a> = (A,)>,
{
    #[doc(hidden)]
    fn call<'a>(&self, (a,): (A,)) -> R {
        self(a)
    }
}

impl<Func, R, A, B> Function<R, (A, B)> for Func
where
    Func: Fn(A, B) -> R,
    R: FunctionReturn,

    A: for<'a> FunctionArg<Output<'a> = A>,
    B: for<'a> FunctionArg<Output<'a> = B>,

    (A, B): for<'a> FunctionArgs<Output<'a> = (A, B)>,
{
    #[doc(hidden)]
    fn call<'a>(&self, (a, b): (A, B)) -> R {
        self(a, b)
    }
}

impl<Func, R, A, B, C> Function<R, (A, B, C)> for Func
where
    Func: Fn(A, B, C) -> R,
    R: FunctionReturn,

    A: for<'a> FunctionArg<Output<'a> = A>,
    B: for<'a> FunctionArg<Output<'a> = B>,
    C: for<'a> FunctionArg<Output<'a> = C>,

    (A, B, C): for<'a> FunctionArgs<Output<'a> = (A, B, C)>,
{
    #[doc(hidden)]
    fn call<'a>(&self, (a, b, c): (A, B, C)) -> R {
        self(a, b, c)
    }
}

impl<Func, R, A, B, C, D> Function<R, (A, B, C, D)> for Func
where
    Func: Fn(A, B, C, D) -> R,
    R: FunctionReturn,

    A: for<'a> FunctionArg<Output<'a> = A>,
    B: for<'a> FunctionArg<Output<'a> = B>,
    C: for<'a> FunctionArg<Output<'a> = C>,
    D: for<'a> FunctionArg<Output<'a> = D>,

    (A, B, C, D): for<'a> FunctionArgs<Output<'a> = (A, B, C, D)>,
{
    #[doc(hidden)]
    fn call<'a>(&self, (a, b, c, d): (A, B, C, D)) -> R {
        self(a, b, c, d)
    }
}

impl<Func, R, A, B, C, D, E> Function<R, (A, B, C, D, E)> for Func
where
    Func: Fn(A, B, C, D, E) -> R,
    R: FunctionReturn,

    A: for<'a> FunctionArg<Output<'a> = A>,
    B: for<'a> FunctionArg<Output<'a> = B>,
    C: for<'a> FunctionArg<Output<'a> = C>,
    D: for<'a> FunctionArg<Output<'a> = D>,
    E: for<'a> FunctionArg<Output<'a> = E>,

    (A, B, C, D, E): for<'a> FunctionArgs<Output<'a> = (A, B, C, D, E)>,
{
    #[doc(hidden)]
    fn call<'a>(&self, (a, b, c, d, e): (A, B, C, D, E)) -> R {
        self(a, b, c, d, e)
    }
}

////////////////////////////////////////////////////////////////////////////////
// FunctionArgs
////////////////////////////////////////////////////////////////////////////////

impl FunctionArgs for () {
    type Output<'a> = ();

    fn from_state<'args>(state: FunctionState<'_, 'args>) -> Result<Self::Output<'args>> {
        let [] = get_args::<0>(state.args)?;
        Ok(())
    }
}

impl<A> FunctionArgs for (A,)
where
    A: FunctionArg,
{
    type Output<'a> = (A::Output<'a>,);

    fn from_state<'args>(state: FunctionState<'_, 'args>) -> Result<Self::Output<'args>> {
        let err = |e, sp| err_expected_arg(e, state.source, state.fname, sp);
        let [(a, sa)] = get_args(state.args)?;
        let a = A::from_value(a).map_err(|e| err(e, *sa))?;
        Ok((a,))
    }
}

impl<A, B> FunctionArgs for (A, B)
where
    A: FunctionArg,
    B: FunctionArg,
{
    type Output<'a> = (A::Output<'a>, B::Output<'a>);

    fn from_state<'args>(state: FunctionState<'_, 'args>) -> Result<Self::Output<'args>> {
        let err = |e, sp| err_expected_arg(e, state.source, state.fname, sp);
        let [(a, sa), (b, sb)] = get_args(state.args)?;
        let a = A::from_value(a).map_err(|e| err(e, *sa))?;
        let b = B::from_value(b).map_err(|e| err(e, *sb))?;
        Ok((a, b))
    }
}

impl<A, B, C> FunctionArgs for (A, B, C)
where
    A: FunctionArg,
    B: FunctionArg,
    C: FunctionArg,
{
    type Output<'a> = (A::Output<'a>, B::Output<'a>, C::Output<'a>);

    fn from_state<'args>(state: FunctionState<'_, 'args>) -> Result<Self::Output<'args>> {
        let err = |e, sp| err_expected_arg(e, state.source, state.fname, sp);
        let [(a, sa), (b, sb), (c, sc)] = get_args(state.args)?;
        let a = A::from_value(a).map_err(|e| err(e, *sa))?;
        let b = B::from_value(b).map_err(|e| err(e, *sb))?;
        let c = C::from_value(c).map_err(|e| err(e, *sc))?;
        Ok((a, b, c))
    }
}

impl<A, B, C, D> FunctionArgs for (A, B, C, D)
where
    A: FunctionArg,
    B: FunctionArg,
    C: FunctionArg,
    D: FunctionArg,
{
    type Output<'a> = (A::Output<'a>, B::Output<'a>, C::Output<'a>, D::Output<'a>);

    fn from_state<'args>(state: FunctionState<'_, 'args>) -> Result<Self::Output<'args>> {
        let err = |e, sp| err_expected_arg(e, state.source, state.fname, sp);
        let [(a, sa), (b, sb), (c, sc), (d, sd)] = get_args(state.args)?;
        let a = A::from_value(a).map_err(|e| err(e, *sa))?;
        let b = B::from_value(b).map_err(|e| err(e, *sb))?;
        let c = C::from_value(c).map_err(|e| err(e, *sc))?;
        let d = D::from_value(d).map_err(|e| err(e, *sd))?;
        Ok((a, b, c, d))
    }
}

impl<A, B, C, D, E> FunctionArgs for (A, B, C, D, E)
where
    A: FunctionArg,
    B: FunctionArg,
    C: FunctionArg,
    D: FunctionArg,
    E: FunctionArg,
{
    type Output<'a> = (
        A::Output<'a>,
        B::Output<'a>,
        C::Output<'a>,
        D::Output<'a>,
        E::Output<'a>,
    );

    fn from_state<'args>(state: FunctionState<'_, 'args>) -> Result<Self::Output<'args>> {
        let err = |e, sp| err_expected_arg(e, state.source, state.fname, sp);
        let [(a, sa), (b, sb), (c, sc), (d, sd), (e, se)] = get_args(state.args)?;
        let a = A::from_value(a).map_err(|e| err(e, *sa))?;
        let b = B::from_value(b).map_err(|e| err(e, *sb))?;
        let c = C::from_value(c).map_err(|e| err(e, *sc))?;
        let d = D::from_value(d).map_err(|e| err(e, *sd))?;
        let e = E::from_value(e).map_err(|e| err(e, *se))?;
        Ok((a, b, c, d, e))
    }
}

fn get_args<'stack, 'args, const N: usize>(
    args: &'args mut [(ValueCow<'stack>, Span)],
) -> Result<&'args mut [(ValueCow<'stack>, Span); N]> {
    let n = args.len();
    args.try_into()
        .map_err(|_| Error::render_plain(format!("function expects {N} arguments, {n} provided")))
}

fn err_expected_arg(err: args::Error, source: &str, fname: &str, span: Span) -> Error {
    let msg = match err {
        args::Error::Type(exp, got) => {
            format!("function `{fname}` expects {exp} argument, found {got}")
        }
        args::Error::TryFromInt(want, value) => {
            format!("function `{fname}` expects {want} argument, but `{value}` is out of range",)
        }
    };
    Error::render(msg, source, span)
}

////////////////////////////////////////////////////////////////////////////////
// FunctionReturn
////////////////////////////////////////////////////////////////////////////////

impl<T> FunctionReturn for T
where
    T: Into<Value>,
{
    fn to_value(self) -> Result<Value> {
        Ok(self.into())
    }
}

impl<T, E> FunctionReturn for std::result::Result<T, E>
where
    T: Into<Value>,
    E: FunctionError,
{
    fn to_value(self) -> Result<Value> {
        self.map(Into::into).map_err(FunctionError::to_error)
    }
}

////////////////////////////////////////////////////////////////////////////////
// FunctionError
////////////////////////////////////////////////////////////////////////////////

impl FunctionError for String {
    fn to_error(self) -> Error {
        Error::function(self)
    }
}

impl FunctionError for &str {
    fn to_error(self) -> Error {
        Error::function(self)
    }
}
