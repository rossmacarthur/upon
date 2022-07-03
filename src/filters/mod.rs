mod args;
mod impls;

use crate::render::{FilterState, Stack};
use crate::types::ast::Arg;
use crate::types::span::Span;
use crate::value::ValueCow;
use crate::{Error, Result, Value};

pub type FilterFn = dyn Fn(FilterState<'_>) -> Result<Value> + Send + Sync + 'static;

pub fn new<F, R, A>(f: F) -> Box<FilterFn>
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
/// This trait is used by the [`Engine::add_filter`][crate::Engine::add_filter]
/// method to abstract over a variety of function and closure types. This
/// includes filters with variable argument types, return types and arity. The
/// first argument to a filter function will always receive the piped value or
/// expression. It can then have up to four more arguments. The renderer will
/// check the number of arguments and the type of arguments at when the filter
/// is used.
///
/// [`Filter`] is implemented for functions that return any of the following
/// types.
///
/// - `R` where `R` implements `Into<Value>`
/// - `Option<R>` where `R` implements `Into<Value>`
/// - `Result<R>` where `R` implements `Into<Value>`
///
/// [`Filter`] is implemented for functions that take any of the following owned
/// types as arguments.
/// - [`bool`]
/// - [`i64`]
/// - [`f64`]
/// - [`String`]
/// - [`Vec<Value>`]
/// - [`BTreeMap<String, Value>`][std::collections::BTreeMap]
/// - [`Value`]
///
/// The first argument can also specified using the following reference types.
/// - [`&str`][str]
/// - [`&[Value]`][Vec<Value>]
/// - [`&BTreeMap<String, Value>`][std::collections::BTreeMap]
/// - [`&Value`][Value]
///
/// Other arguments can also use [`&str`][str] if the value passed in is a
/// literal.
///
/// ## Examples
///
/// Consider the following template.
///
/// ```text
/// {{ user.name | split: " " | last }}
/// ```
///
/// We could implement the `split` and `last` filters like this:
///
/// ```rust
/// use upon::{Engine, Value};
///
/// let mut engine = Engine::new();
/// engine.add_filter("split", split);
/// engine.add_filter("last", last);
///
/// fn split(s: &str, sep: &str) -> Vec<String> {
///     s.split(sep).map(String::from).collect()
/// }
///
/// fn last(mut list: Vec<Value>) -> Option<Value> {
///     list.pop()
/// }
/// ```
pub trait Filter<R, A>
where
    A: for<'a> FilterArgs<'a>,
{
    #[doc(hidden)]
    fn filter(&self, args: <A as FilterArgs<'_>>::Output) -> R;
}

pub trait FilterArgs<'a> {
    type Output: 'a;
    fn from_state(state: FilterState<'a>) -> Result<Self::Output>;
}

pub trait FilterArg<'a> {
    type Output: 'a;
    fn from_value(v: Value) -> args::Result<Self::Output>;
    fn from_value_ref(v: &'a Value) -> args::Result<Self::Output>;
    fn from_cow_mut(v: &'a mut ValueCow<'a>) -> args::Result<Self::Output>;
}

pub trait FilterReturn {
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
        let err = |e| err_expected_val(e, state.stack.source(), state.filter.span);
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
        let err = |e| err_expected_val(e, state.stack.source(), state.filter.span);
        let v = V::from_cow_mut(state.value).map_err(err)?;
        let a = get_arg::<A>(state.stack, state.args, 0)?;
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
        let err = |e| err_expected_val(e, state.stack.source(), state.filter.span);
        let v = V::from_cow_mut(state.value).map_err(err)?;
        let a = get_arg::<A>(state.stack, state.args, 0)?;
        let b = get_arg::<B>(state.stack, state.args, 1)?;
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
        let err = |e| err_expected_val(e, state.stack.source(), state.filter.span);
        let v = V::from_cow_mut(state.value).map_err(err)?;
        let a = get_arg::<A>(state.stack, state.args, 0)?;
        let b = get_arg::<B>(state.stack, state.args, 1)?;
        let c = get_arg::<C>(state.stack, state.args, 2)?;
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
        let err = |e| err_expected_val(e, state.stack.source(), state.filter.span);
        let v = V::from_cow_mut(state.value).map_err(err)?;
        let a = get_arg::<A>(state.stack, state.args, 0)?;
        let b = get_arg::<B>(state.stack, state.args, 1)?;
        let c = get_arg::<C>(state.stack, state.args, 2)?;
        let d = get_arg::<D>(state.stack, state.args, 3)?;
        Ok((v, a, b, c, d))
    }
}

fn check_args(state: &FilterState<'_>, exp: usize) -> Result<()> {
    if state.args.len() == exp {
        Ok(())
    } else {
        Err(Error::new(
            format!("filter expected {} arguments", exp),
            state.stack.source(),
            state.filter.span,
        ))
    }
}

fn get_arg<'a, T>(stack: &'a Stack<'a, 'a>, args: &'a [Arg<'a>], i: usize) -> Result<T::Output>
where
    T: FilterArg<'a>,
{
    match &args[i] {
        Arg::Var(var) => match stack.resolve_path(&var.path)? {
            ValueCow::Borrowed(v) => {
                T::from_value_ref(v).map_err(|e| err_expected_arg(e, stack.source(), var.span))
            }
            ValueCow::Owned(v) => {
                T::from_value(v).map_err(|e| err_expected_arg(e, stack.source(), var.span))
            }
        },
        Arg::Literal(lit) => {
            T::from_value_ref(&lit.value).map_err(|e| err_expected_arg(e, stack.source(), lit.span))
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
    Error::new(msg, source, span)
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
    Error::new(msg, source, span)
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
