#[test]
fn address_schema() {
    let schema = include_str!("../../schema-examples/address.schema.json");
    crate::schema2print(schema);
}

#[test]
fn calendar_schema() {
    let schema = include_str!("../../schema-examples/calendar.schema.json");
    crate::schema2print(schema);
}

#[test]
fn card_schema() {
    let schema = include_str!("../../schema-examples/card.schema.json");
    crate::schema2print(schema);
}

#[test]
fn geographical_location_schema() {
    let schema = include_str!("../../schema-examples/geographical-location.schema.json");
    crate::schema2print(schema);
}

#[test]
fn vega_schema() {
    let schema = include_str!("../../schema-examples/vega-v5.schema.json");
    crate::schema2print(schema);
}

#[test]
fn vega_lite_schema() {
    let schema = include_str!("../../schema-examples/vega-lite-v5.schema.json");
    crate::schema2print(schema);
}
