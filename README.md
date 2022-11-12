<!-- generated by tools/gen-readme -->

# upon

[![Crates.io Version](https://img.shields.io/crates/v/upon.svg)](https://crates.io/crates/upon)
[![Docs.rs Latest](https://img.shields.io/badge/docs.rs-latest-blue.svg)](https://docs.rs/upon)
[![Build Status](https://img.shields.io/github/workflow/status/rossmacarthur/upon/build/trunk)](https://github.com/rossmacarthur/upon/actions?query=workflow%3Abuild)

A lightweight and powerful template engine for Rust.

## Table of Contents

- [Overview](#overview)
  - [Syntax](#syntax)
  - [Engine](#engine)
  - [Why another template engine?](#why-another-template-engine)
  - [MSRV](#msrv)
- [Getting started](#getting-started)
- [Further reading](#further-reading)
- [Features](#features)
- [Benchmarks](#benchmarks)
- [License](#license)

## Overview

### Syntax

- Expressions: `{{ user.name }}`
- Conditionals: `{% if user.enabled %} ... {% endif %}`
- Loops: `{% for user in users %} ... {% endfor %}`
- Nested templates: `{% include "nested" %}`
- Configurable delimiters: `<? user.name ?>`, `(( if user.enabled ))`
- Arbitrary user defined filters: `{{ user.name | replace: "\t", " " }}`

### Engine

- Clear and well documented API
- Customizable value formatters: `{{ user.name | escape_html }}`
- Render to a [`String`](https://doc.rust-lang.org/stable/std/string/struct.String.html) or any [`std::io::Write`](https://doc.rust-lang.org/stable/std/io/trait.Write.html) implementor
- Render using any [`serde`](https://crates.io/crates/serde) serializable values
- Convenient macro for quick rendering:
  `upon::value!{ name: "John", age: 42 }`
- Pretty error messages when displayed using `{:#}`
- Format agnostic (does *not* escape values for HTML by default)
- Minimal dependencies and decent runtime performance

### Why another template engine?

It’s true there are already a lot of template engines for Rust!

I created `upon` because I required a template engine that had runtime
compiled templates, configurable syntax delimiters and minimal dependencies.
I also didn’t need support for arbitrary expressions in the template syntax
but occasionally I needed something more flexible than outputting simple
values (hence filters). Performance was also a concern for me, template
engines like [Handlebars] and [Tera] have a lot of features but can be up to
five to seven times slower to render than engines like [TinyTemplate].

Basically I wanted something like [TinyTemplate] with support for
configurable delimiters and user defined filter functions. The syntax is
inspired by template engines like [Liquid] and [Jinja].

### MSRV

Currently the minimum supported version for `upon` is Rust 1.60. The policy
of this crate is to only increase the MSRV in a breaking release.

## Getting started

First, add the crate to your Cargo manifest.

```sh
cargo add upon
```

Now construct an [`Engine`](https://docs.rs/upon/latest/upon/struct.Engine.html). The engine stores the syntax config, filter
functions, formatters, and compiled templates. Generally, you only need to
construct one engine during the lifetime of a program.

```rust
let engine = upon::Engine::new();
```

Next, [`add_template`](https://docs.rs/upon/latest/upon/struct.Engine.html#method.add_template) is used to compile and store a
template in the engine.

```rust
engine.add_template("hello", "Hello {{ user.name }}!")?;
```

Finally, the template is rendered by fetching it using
[`get_template`](https://docs.rs/upon/latest/upon/struct.Engine.html#method.get_template) and calling
[`render`](https://docs.rs/upon/latest/upon/struct.TemplateRef.html#method.render).

```rust
let template = engine.get_template("hello").unwrap();
let result = template.render(upon::value!{ user: { name: "John Smith" }})?;
assert_eq!(result, "Hello John Smith!");
```

If the lifetime of the template source is shorter than the engine lifetime
or you don’t need to store the compiled template then you can also use the
[`compile`](https://docs.rs/upon/latest/upon/struct.Engine.html#method.compile) function to return the template directly.

```rust
let template = engine.compile("Hello {{ user.name }}!")?;
let result = template.render(upon::value!{ user: { name: "John Smith" }})?;
assert_eq!(result, "Hello John Smith!");
```

## Further reading

- The [`syntax`](./SYNTAX.md) module documentation outlines the template syntax.
- The [`filters`](https://docs.rs/upon/latest/upon/filters/index.html) module documentation describes filters and how they work.
- The [`fmt`](https://docs.rs/upon/latest/upon/fmt/index.html) module documentation contains information on value formatters.
- The [`examples/`](https://github.com/rossmacarthur/upon/tree/trunk/examples) directory in the repository contains concrete
  code examples.

## Features

The following crate features are available.

- **`filters`** *(enabled by default)* — Enables support for filters in
  templates (see [`Engine::add_filter`](https://docs.rs/upon/latest/upon/struct.Engine.html#method.add_filter)). This does *not* affect value
  formatters (see [`Engine::add_formatter`](https://docs.rs/upon/latest/upon/struct.Engine.html#method.add_formatter)). Disabling this will improve
  compile times.

- **`serde`** *(enabled by default)* — Enables all serde support and pulls
  in the [`serde`](https://crates.io/crates/serde) crate as a dependency. If disabled then you can use
  [`render_from`](https://docs.rs/upon/latest/upon/struct.TemplateRef.html#method.render_from) to render templates and
  construct the context using [`Value`](https://docs.rs/upon/latest/upon/enum.Value.html)’s `From` impls.

- **`unicode`** *(enabled by default)* — Enables unicode support and pulls
  in the [`unicode-ident`](https://crates.io/crates/unicode-ident) and
  [`unicode-width`](https://crates.io/crates/unicode-width) crates. If disabled then unicode
  identifiers will no longer be allowed in templates and `.chars().count()`
  will be used in error formatting.

To disable all features or to use a subset you need to set `default-features = false` in your Cargo manifest and then enable the features that you would
like. For example to use **`serde`** but disable **`filters`** and
**`unicode`** you would do the following.

```toml
[dependencies]
upon = { version = "...", default-features = false, features = ["serde"] }
```

[Handlebars]: https://crates.io/crates/handlebars
[Tera]: https://crates.io/crates/tera
[TinyTemplate]: https://crates.io/crates/tinytemplate
[TinyTemplate]: https://crates.io/crates/tinytemplate
[Liquid]: https://liquidjs.com
[Jinja]: https://jinja.palletsprojects.com

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
