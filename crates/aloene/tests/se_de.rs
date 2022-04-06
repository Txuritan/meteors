use aloene::{test_utils::se_de, Aloene};

#[derive(Debug, PartialEq, Aloene)]
struct Childless {
    field1: String,
    field2: EnumUnit,
    field3: bool,
}

#[derive(Debug, PartialEq, Aloene)]
enum EnumUnit {
    Yes,
    No,
}

#[test]
fn test_childless_unit() {
    se_de(Childless {
        field1: "Hello, World! How are you?".into(),
        field2: EnumUnit::Yes,
        field3: true,
    });

    se_de(Childless {
        field1: "Hello, World! How are you?".into(),
        field2: EnumUnit::No,
        field3: true,
    });
}
