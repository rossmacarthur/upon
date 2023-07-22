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
    let ctx = Context {
        user: User {
            name: "John Smith".into(),
        },
    };

    let output = upon::Engine::new()
        .compile("Hello {{ user.name }}!")?
        .render(ctx)
        .to_string()?;

    println!("{output}");

    Ok(())
}
