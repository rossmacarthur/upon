fn main() -> upon::Result<()> {
    let engine = upon::Engine::new();

    let out = engine
        .compile("Hello {{ name }}!")?
        .render(&engine, upon::value! { name: "World" })
        .to_string()?;

    println!("{out}");

    Ok(())
}
