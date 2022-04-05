# upon

A tiny, configurable find-and-replace template engine.

## Features

- Rendering values, e.g. `{{ user.name }}`
- Configurable template tags, e.g. `<? value ?>`
- Arbitrary filter functions to transform data, e.g. `{{ value | my_filter }}`

 ## Examples

 #### Render data constructed using the macro

```rust
use upon::{Engine, data};

let result = Engine::new()
   .compile("Hello {{ value }}")?
   .render(data! { value: "World!" })?;

assert_eq!(result, "Hello World!");
```

### Render a template using custom tags

```rust
use upon::{data, Engine};

let result = Engine::with_tags("<?", "?>")
   .compile("Hello <? value ?>")?
   .render(data! { value: "World!" })?;

assert_eq!(result, "Hello World!");
```

### Render using structured data

```rust
use upon::Engine;
use serde::Serialize;

#[derive(Serialize)]
struct Data {
   user: User,
}
#[derive(Serialize)]
struct User {
   name: String,
}

let data = Data { user: User { name: "John Smith".into() } };

let result = Engine::new().compile("Hello {{ user.name }}")?.render(data)?;

assert_eq!(result, "Hello John Smith");
```

### Transform data using filters

```rust
use upon::{data, Engine, Value};

fn lower(mut v: Value) -> Value {
   if let Value::String(s) = &mut v {
       *s = s.to_lowercase();
   }
   v
}

let mut engine = Engine::new();
engine.add_filter("lower", lower);

let result = engine
   .compile("Hello {{ value | lower }}")?
   .render(data! { value: "WORLD!" })?;

assert_eq!(result, "Hello world!");
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
