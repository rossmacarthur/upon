use upon::Value;

fn main() -> upon::Result<()> {
    let mut engine = upon::Engine::new();

    // Any functions with the supported signatures can be used
    engine.add_filter("lower", str::to_lowercase);

    // Filters can be closures
    engine.add_filter("contains", |s: &str, other: &str| -> bool {
        s.contains(other)
    });

    // Filters can be free functions
    engine.add_filter("is_empty", is_empty);

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

    let t = engine.get_template("example").unwrap();

    println!(
        "# case 1\n{}",
        t.render(upon::value! { user: { name: "" } })?
    );

    println!(
        "# case 2\n{}",
        t.render(upon::value! { user: { name: "John Smith" } })?
    );

    println!(
        "# case 3\n{}",
        t.render(upon::value! { user: { name: "Jane Doe" } })?
    );

    Ok(())
}

/// This filter takes value by reference so that the renderer doesn't have to
/// clone the value before passing it to the filter.
fn is_empty(value: &Value) -> Result<bool, String> {
    match value {
        Value::String(v) => Ok(v.is_empty()),
        Value::List(v) => Ok(v.is_empty()),
        Value::Map(v) => Ok(v.is_empty()),
        v => Err(format!("unsupported type {:?}", v)),
    }
}
