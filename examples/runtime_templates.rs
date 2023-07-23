use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> upon::Result<()> {
    // The *runtime* directory that the templates will be loaded from.
    let template_dir = PathBuf::from_iter([env!("CARGO_WORKSPACE_DIR"), "examples", "templates"]);

    let mut engine = upon::Engine::new();
    add_templates(&mut engine, &template_dir)?;

    // Render the template using the provided data
    let output = engine
        .template("index")
        .render(upon::value! { title: "My Webpage!", year: 2022 })
        .to_string()?;

    println!("{output}");

    Ok(())
}

/// Adds all HTML templates in the given directory to the engine, using the
/// file name as the template name. This implementation does not recurse
/// directories.
fn add_templates(engine: &mut upon::Engine<'_>, dir: &Path) -> upon::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        if path.extension() != Some(OsStr::new("html")) {
            continue;
        }

        // Converts a file name like 'templates/index.html' to the name 'index'
        let name = path
            .file_stem()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();

        // Reads the template source
        let text = fs::read_to_string(&path)?;

        // Compiles and add the template to the engine!
        engine.add_template(name, text)?;
    }
    Ok(())
}
