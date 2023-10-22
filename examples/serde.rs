use serde::Serialize;

#[derive(Serialize)]
struct Context {
    user: User,
}

#[derive(Serialize)]
struct User {
    name: String,
}

fn main() -> upon::Result<()> {
    let engine = upon::Engine::new();

    let ctx = Context {
        user: User {
            name: "John Smith".into(),
        },
    };

    let output = engine
        .compile("Hello {{ user.name }}!")?
        .render(&engine, ctx)
        .to_string()?;

    println!("{output}");

    Ok(())
}
