# upon

A tiny, configurable find-and-replace template engine.

## Features

- Rendering values, e.g. `{{ path.to.value }}`
- Configurable template tags, e.g. `<? value ?>`
- Arbitrary filter functions, e.g. `{{ value | filter }}`

## Examples

Render data constructed using the macro.

```rust
use upon::data;

let result = upon::render("Hello {{ value }}", data! { value: "World!" })?;
assert_eq!(result, "Hello World!");
```

Render using structured data.

```rust
#[derive(serde::Serialize)]
struct Data {
    value: String
}

let result = upon::render("Hello {{ value }}", Data { value: "World!".into() })?;
assert_eq!(result, "Hello World!");
```

Render a template using custom tags.

```rust
use upon::{data, Engine};

let engine = Engine::with_tags("<?", "?>");
let result = engine.render("Hello <? value ?>", data! { value: "World!" })?;
assert_eq!(result, "Hello World!");
```

Transform data using filters.

```rust
use upon::{data, Engine, Value};

let mut engine = Engine::new();
engine.add_filter("lower", |mut v| {
    if let Value::String(s) = &mut v {
       *s = s.to_lowercase();
    }
    v
});

let result = engine.render("Hello {{ value | lower }}", data! { value: "WORLD!" })?;
assert_eq!(result, "Hello world!");
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
