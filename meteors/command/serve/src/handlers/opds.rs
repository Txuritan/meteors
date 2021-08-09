use {std::str::FromStr, common::{database::Database, prelude::*}, enrgy::web};

pub enum CatalogueFormat {
    Atom, // opds+1.2
    Html,
    Json, // opds+2.0
}

impl FromStr for CatalogueFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "atom" => Ok(CatalogueFormat::Atom),
            "html" => Ok(CatalogueFormat::Html),
            "json" => Ok(CatalogueFormat::Json),
            ext => Err(anyhow!("Unknown catalogue format: {}", ext)),
        }
    }
}

pub fn catalogue_handler(db: web::Data<Database>, ext: web::ParseParam<"ext", CatalogueFormat>) {}
