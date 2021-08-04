# zoinks

`Zoinks` is a parser generator.  It takes [JSON Schema](https://json-schema.org/) (draft 7) files and generates Rust enums and struct that will (using [`serde`](https://github.com/serde-rs/serde/)) parse JSON matching the schema.  JSON Schemas can be exceptionally complex and `zoinks` does not aim to support all of the nuances.  The initial goal is to parse just enough of the JSON Schema spec to quickly create a parser for [Vega Lite](https://vega.github.io/vega-lite/) charting files.  If something is not parsing or validating correctly that may be by design.  Pull requests welcomed.

There are two ways to use `zoinks`:

### As a command line utility
```ShellSession
$ cat test.schema.json
{
    "$id": "Test Schema",
    "$schema": "http://json-schema.org/draft-07/schema#",
    "definitions": {
        "ExprRef": {
            "additionalProperties": false,
            "properties": {
              "expr": {
                "description": "Vega expression (which can refer to Vega-Lite parameters).",
                "type": "string"
              }
            },
            "required": [
              "expr"
            ],
            "type": "object"
          },
          "angle": {
            "anyOf": [
              {
                "description": "The rotation angle of the text, in degrees.",
                "maximum": 360,
                "minimum": 0,
                "type": "number"
              },
              {
                "$ref": "#/definitions/ExprRef"
              }
            ]
          }
    }
}
$ cargo run -- -i test.schema.json > parser.rs
    Finished dev [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/print-bindings -i test-parse/test.schema.json`
[2021-08-04T01:00:56Z INFO  zoinks_lib] Reading file
[2021-08-04T01:00:56Z INFO  zoinks_lib] Parsing JSON
[2021-08-04T01:00:56Z INFO  zoinks_lib] Generating structs
[2021-08-04T01:00:56Z INFO  zoinks_lib] Done
$ echo $?
0
$ rustfmt < foo.rs
#[derive(Debug, serde :: Deserialize)]
pub struct Null;
#[derive(Debug, serde :: Deserialize)]
pub struct ExprRefPrptyExpr(String);
#[derive(Debug, serde :: Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExprRef {
    #[doc = "Vega expression (which can refer to Vega-Lite parameters)."]
    pub expr: ExprRefPrptyExpr,
}
#[derive(Debug, serde :: Deserialize)]
pub struct AngleNumber0(f64);
#[doc = "any_of enum: Angle"]
#[derive(Debug, serde :: Deserialize)]
#[serde(untagged)]
pub enum Angle {
    AngleNumber0(AngleNumber0),
    ExprRef(ExprRef),
}
```

### As a codegen macro

For example this could be used inside `build.rs`.

```rust
use zoinks_codegen::schema2struct;

// This macro will expand to the above structs
schema2struct!("test.schema.json")
```
