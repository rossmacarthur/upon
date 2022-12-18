fn main() -> upon::Result<()> {
    let mut engine = upon::Engine::new();
    engine.add_template("index", include_str!("templates/index.html"))?;
    engine.add_template("header", include_str!("templates/header.html"))?;
    engine.add_template("footer", include_str!("templates/footer.html"))?;

    // Fetch the template with name "index"
    let template = engine.get_template("index").unwrap();

    // Render the template using the provided data
    let output = template.render(upon::value! { title: "My Webpage!", year: 2022 })?;

    println!("{output}");

    Ok(())
}
