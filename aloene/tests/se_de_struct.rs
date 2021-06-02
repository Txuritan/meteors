use aloene::{test_utils::se_de, Aloene};

#[derive(Debug, PartialEq, Aloene)]
struct Childless {
    field1: String,
    field2: u32,
    field3: bool,
}

#[test]
fn test_childless() {
    se_de(Childless {
        field1: "Hello, World! How are you?".into(),
        field2: 6543,
        field3: true,
    });
}

#[derive(Debug, PartialEq, Aloene)]
struct Children {
    field1: bool,
    field2: f32,
    field3: Childless,
}

#[test]
fn test_children() {
    se_de(Children {
        field1: false,
        field2: 5432.2,
        field3: Childless {
            field1: "Hello, World! How are you?".into(),
            field2: 6543,
            field3: true,
        },
    });
}

#[derive(Debug, PartialEq, Aloene)]
struct Optional {
    field1: bool,
    field2: f32,
    field3: Option<Childless>,
}

#[test]
fn test_optional() {
    se_de(Optional {
        field1: false,
        field2: 5432.2,
        field3: Some(Childless {
            field1: "Hello, World! How are you?".into(),
            field2: 6543,
            field3: true,
        }),
    });

    se_de(Optional {
        field1: false,
        field2: 5432.2,
        field3: None,
    });
}
