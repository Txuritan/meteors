pub struct Container;

impl Container {
    pub const UNIT: u8 = 0x00;

    pub const NONE: u8 = 0x01;
    pub const SOME: u8 = 0x02;

    pub const VALUE: u8 = 0x03;

    pub const VARIANT: u8 = 0x04;

    pub const STRUCT: u8 = 0x05;
    pub const ARRAY: u8 = 0x06;

    pub const MAP: u8 = 0x07;
    pub const LIST: u8 = 0x08;
}

pub struct Value;

impl Value {
    pub const BOOL: u8 = 0x00;
    pub const STRING: u8 = 0x01;

    pub const FLOAT_32: u8 = 0x10;
    pub const FLOAT_64: u8 = 0x11;

    pub const SIGNED_8: u8 = 0x20;
    pub const SIGNED_16: u8 = 0x21;
    pub const SIGNED_32: u8 = 0x22;
    pub const SIGNED_64: u8 = 0x23;
    pub const SIGNED_SIZE: u8 = 0x24;

    pub const UNSIGNED_8: u8 = 0x30;
    pub const UNSIGNED_16: u8 = 0x31;
    pub const UNSIGNED_32: u8 = 0x32;
    pub const UNSIGNED_64: u8 = 0x33;
    pub const UNSIGNED_SIZE: u8 = 0x34;
}
