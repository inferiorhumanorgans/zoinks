// Parses a JSON schema definition to Rust objects
mod parser;
pub use parser::Schema;

// Generates rust objects to parse an implementation of a specific schema
mod generator;
pub use generator::genimpl;

use log::warn;

// Reads a JSON schema definition in and prints parser objects to stdout
pub fn schema2print(input_fn: &str) -> String {
    warn!("Reading file");
    let schema_string = std::fs::read_to_string(input_fn).unwrap();

    warn!("Parsing JSON");
    let schema = serde_json::from_str::<Schema>(&schema_string).unwrap();

    warn!("Generating structs");
    let imp = genimpl(&schema);
    warn!("Done");

    format!("{:#}", imp)
}

#[cfg(test)]
mod test;
