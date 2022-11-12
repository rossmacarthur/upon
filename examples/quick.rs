fn main() -> upon::Result<()> {
    let out = upon::Engine::new()
        .compile("Hello {{ name }}!")?
        .render(upon::value! { name: "World" })?;

    println!("{}", out);

    Ok(())
}
