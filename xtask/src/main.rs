use std::path::PathBuf;

#[derive(serde::Deserialize)]
struct Template {
    root: String,
    bounds_impl: Option<String>,
    bounds_name: Option<String>,
    name: String,
    path: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);

    match args.next().as_deref() {
        Some("codegen") => {
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .delimiter(b';')
                .from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/templates.csv"))?;

            for template in rdr.deserialize() {
                let template: Template = template?;

                let root_path = PathBuf::from(template.root);
                let input_path = root_path.join("templates").join(&template.path);

                let input = template.path.replace('-', "_");

                let mut output_path = root_path.join("src").join("templates").join(input);
                output_path.set_extension("hbs.rs");

                std::fs::create_dir_all(output_path.parent().unwrap())?;

                let bounds = template
                    .bounds_impl
                    .as_deref()
                    .zip(template.bounds_name.as_deref());

                let input_content = std::fs::read_to_string(input_path)?;
                let output_content = opal::compile(bounds, &template.name, &input_content)?;

                std::fs::write(output_path, output_content)?;
            }
        }
        cmd => println!("unknown command `{:?}`", cmd),
    }

    Ok(())
}
