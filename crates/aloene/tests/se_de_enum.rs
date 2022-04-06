use aloene::{test_utils::se_de, Aloene};

#[derive(Debug, PartialEq, Aloene)]
enum EnumUnit {
    Yes,
    No,
}

#[test]
fn test_unit() {
    se_de(EnumUnit::Yes);
    se_de(EnumUnit::No);
}
