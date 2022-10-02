<!-- generated by tools/gen-readme -->

# upon

[![Crates.io Version](https://img.shields.io/crates/v/upon.svg)](https://crates.io/crates/upon)
[![Docs.rs Latest](https://img.shields.io/badge/docs.rs-latest-blue.svg)](https://docs.rs/upon)
[![Build Status](https://img.shields.io/github/workflow/status/rossmacarthur/upon/build/trunk)](https://github.com/rossmacarthur/upon/actions?query=workflow%3Abuild)

A simple, powerful template engine.

## Features

#### Syntax

- Expressions: `{{ user.name }}`
- Conditionals: `{% if user.enabled %} ... {% endif %}`
- Loops: `{% for user in users %} ... {% endfor %}`
- Nested templates: `{% include "nested" %}`
- Configurable delimiters: `<? user.name ?>`, `(( if user.enabled ))`
- Arbitrary user defined filters: `{{ user.name | replace: "\t", " " }}`

#### Engine

- Clear and well documented API
- Customizable value formatters: `{{ user.name | escape_html }}`
- Render to a `String` or any `std::io::Write` implementor
- Render using any `serde` serializable values
- Convenient macro for quick rendering:
  `upon::value!{ name: "John", age: 42 }`
- Minimal dependencies and decent runtime performance

## Getting started

Your entry point is the `Engine` struct. The engine stores the syntax
config, filter functions, and compiled templates. Generally, you only need
to construct one engine during the lifetime of a program.

```rust
let engine = upon::Engine::new();
```

Next, `.add_template` is used to compile and store a
template in the engine.

```rust
engine.add_template("hello", "Hello {{ user.name }}!")?;
```

Finally, the template is rendered by fetching it using
`.get_template` and calling
`.render`.

```rust
let template = engine.get_template("hello").unwrap();
let result = template.render(upon::value!{ user: { name: "John Smith" }})?;
assert_eq!(result, "Hello John Smith!");
```

If the lifetime of the template source is shorter than the engine lifetime
or you don’t need to store the compiled template then you can also use the
`.compile` function to return the template directly.

```rust
let template = engine.compile("Hello {{ user.name }}!")?;
let result = template.render(upon::value!{ user: { name: "John Smith" }})?;
assert_eq!(result, "Hello John Smith!");
```

## Features

- **filters** *(enabled by default)* — Enables support for filters in
  templates, see `Engine::add_filter()`. This does *not* affect value
  formatters, see `Engine::add_formatter()`. Disabling this will improve
  compile times.

- **serde** *(enabled by default)* — Enables all serde support and pulls in
  the `serde` crate as a dependency. If disabled then you can use
  `.render_from()` to render templates and
  construct the context using `Value`’s `From` impls.

- **unicode** *(enabled by default)* — Enables improved error formatting
  using the `unicode-width` crate. If disabled then
  `.chars().count()` will be used instead.

## Examples

The following section contains some simple examples. See the
[`examples/`](https://github.com/rossmacarthur/upon/tree/trunk/examples) directory in the repository for more.

#### Render using structured data

You can render using any `serde` serializable data.

```rust
#[derive(serde::Serialize)]
struct Context { user: User }

#[derive(serde::Serialize)]
struct User { name: String }

let ctx = Context { user: User { name: "John Smith".into() } };

let result = upon::Engine::new()
    .compile("Hello {{ user.name }}")?
    .render(&ctx)?;

assert_eq!(result, "Hello John Smith");
```

#### Transform data using filters

Data can be transformed using registered filters.

```rust
let mut engine = upon::Engine::new();
engine.add_filter("lower", str::to_lowercase);

let result = engine
    .compile("Hello {{ value | lower }}")?
    .render(upon::value! { value: "WORLD!" })?;

assert_eq!(result, "Hello world!");
```

See the `Filter` trait documentation for more information on filters.

#### Render a template using custom syntax

The template syntax can be set by constructing an engine using
`Engine::with_syntax`.

```rust
let syntax = upon::Syntax::builder().expr("<?", "?>").block("<%", "%>").build();

let result = upon::Engine::with_syntax(syntax)
    .compile("Hello <? user.name ?>")?
    .render(upon::value!{ user: { name: "John Smith" }})?;

assert_eq!(result, "Hello John Smith");
```

#### Render a template to an `impl io::Write`

You can render a template directly to a buffer implementing `io::Write`
by using `.render_to_writer()`.

```rust
use std::io;

let stdout = io::BufWriter::new(io::stdout());

upon::Engine::new()
    .compile("Hello {{ user.name }}")?
    .render_to_writer(stdout, upon::value! { user: { name: "John Smith" }})?;
```

#### Add and use a custom formatter

You can add your own custom formatter’s or even override the default
formatter using `Engine::set_default_formatter`. The following example
shows how you could add `debug` formatter to the engine.

```rust
use std::fmt::Write;
use upon::{Formatter, Value, Result};

let mut engine = upon::Engine::new();
engine.add_formatter("debug", |f, value| {
    write!(f, "Value::{:?}", value)?;
    Ok(())
});


let result = engine
    .compile("User age: {{ user.age | debug }}")?
    .render(upon::value! { user: { age: 23 } })?;

assert_eq!(result, "User age: Value::Integer(23)");
```

## Benchmarks

`upon` was benchmarked against several popular template rendering engines in the
Rust ecosystem. Obviously, each of these engines has a completely different
feature set so the benchmark just compares the performance of some of the
features that they share. Handlebars is so slow that it is excluded from the
compile violin plot.

- [handlebars](https://crates.io/crates/handlebars)
- [liquid](https://crates.io/crates/liquid)
- [minijinja](https://crates.io/crates/minijinja)
- [tera](https://crates.io/crates/tera)
- [tinytemplate](https://crates.io/crates/tinytemplate)

![Violin plot of compile results](./benches/results/compile.svg)
![Violin plot of render results](./benches/results/render.svg)

Benchmarking was done using [criterion](https://crates.io/crates/criterion) on
a quiet cloud machine.

**Host**

- Vultr.com
- 4 CPU
- 8192 MB RAM
- Ubuntu 20.04
- Rust 1.64.0

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
