// Parses a JSON schema definition to Rust objects
mod parser;
pub use parser::Schema;

// Generates rust objects to parse an implementation of a specific schema
mod generator;
pub use generator::genimpl;

// Reads a JSON schema definition in and prints parser objects to stdout
pub fn schema2print(input_fn: &str) -> String {
    let schema_string = std::fs::read_to_string(input_fn).unwrap();

    let schema = serde_json::from_str::<Schema>(&schema_string).unwrap();
    let imp = genimpl(&schema);

    format!("{:#}", imp)
}
