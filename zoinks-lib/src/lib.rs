// Parses a JSON schema definition to Rust objects
mod parser;
pub use parser::Schema;

// Generates rust objects to parse an implementation of a specific schema
mod generator;
pub use generator::genimpl;

#[allow(unused)]
use log::{error, warn, info, debug, trace};

// Reads a JSON schema definition in and prints parser objects to stdout
pub fn schemafile2print(input_fn: &str) -> String {
    info!("Reading file");
    let schema_string = std::fs::read_to_string(input_fn).unwrap();

    schema2print(&schema_string)
}

// Reads a JSON schema definition in and prints parser objects to stdout
pub fn schema2print(input_str: &str) -> String {
    info!("Parsing JSON");
    let schema = serde_json::from_str::<Schema>(&input_str).unwrap();

    info!("Generating structs");
    let imp = genimpl(&schema);

    info!("Done");

    format!("{:#}", imp)
}

#[cfg(test)]
mod test;
