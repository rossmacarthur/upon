use upon::Value;

fn main() -> upon::Result<()> {
    let mut engine = upon::Engine::new();

    // Any functions with the supported signatures can be used
    engine.add_function("lower", str::to_lowercase);

    // Functions can be closures
    engine.add_function("contains", |s: &str, other: &str| -> bool {
        s.contains(other)
    });

    // Functions can be free functions
    engine.add_function("is_empty", is_empty);

    engine.add_template(
        "example",
        r#"
{% if user.name | is_empty %}

    No name given

{% else if user.name | contains: "John" %}

    {{ user.name }}

{% else %}

    {{ user.name | lower }}

{% endif %}
"#,
    )?;

    println!(
        "# case 1\n{}",
        engine
            .template("example")
            .render(upon::value! { user: { name: "" } })
            .to_string()?
    );

    println!(
        "# case 2\n{}",
        engine
            .template("example")
            .render(upon::value! { user: { name: "John Smith" } })
            .to_string()?
    );

    println!(
        "# case 3\n{}",
        engine
            .template("example")
            .render(upon::value! { user: { name: "Jane Doe" } })
            .to_string()?
    );

    Ok(())
}

/// This function takes value by reference so that the renderer doesn't have to
/// clone the value before passing it to the function.
fn is_empty(value: &Value) -> Result<bool, String> {
    match value {
        Value::String(v) => Ok(v.is_empty()),
        Value::List(v) => Ok(v.is_empty()),
        Value::Map(v) => Ok(v.is_empty()),
        v => Err(format!("unsupported type {v:?}")),
    }
}
