# aloene

A simple structured binary file format with a derive macro.

## Examples

```rust
use aloene::Aloene;

#[derive(Aloene)]
struct File {
    text: String,
}

fn main() {
    let file = File { text: "Hello World!".to_string() };

    let mut bytes = Vec::new();

    file.serialize(&mut bytes).unwrap();

    std::fs::write("./file.aloe", bytes).unwrap();
}
```

Output as hex.

```
05 01 04 74 65 78 74 03 01 0C 48 65 6C 6C 6F 20 57 6F 72 6C 64 21
```

Output hex explained.

```
05    struct container bytes
01    string value byte
04    string length
74 65 78 74    string data: "text" (the name of the field)
03    struct container field byte
01    string value byte
0C    string length
48 65 6C 6C 6F 20 57 6F 72 6C 64 21 string data: "Hello World!" (the value of the field)
```

## Format

Aloene is split into two 'types' `container` and `value`.
`container`s are things like structs, enums, maps and lists.
While `value`s are things like booleans, numbers, and strings.

### Container Bytes

|    type | byte (hex) | description                                |
| ------: | :--------: | :----------------------------------------- |
|    unit |    0x00    | an empty unit                              |
|    none |    0x01    | a empty nullable `container` or `value`    |
|    some |    0x02    | a existing nullable `container` or `value` |
|   value |    0x03    |                                            |
| variant |    0x04    |                                            |
|  struct |    0x05    |                                            |
|   array |    0x06    |                                            |
|     map |    0x07    |                                            |
|    list |    0x08    |                                            |

### Value Bytes

|                        type | byte (hex) | description |
| --------------------------: | :--------: | :---------- |
|                        bool |    0x00    |             |
|                      string |    0x01    |             |
|                    float 32 |    0x10    |             |
|                    float 64 |    0x11    |             |
|        signed 8 bit integer |    0x20    |             |
|       signed 16 bit integer |    0x21    |             |
|       signed 32 bit integer |    0x22    |             |
|       signed 64 bit integer |    0x23    |             |
|   signed system bit integer |    0x24    |             |
|      unsigned 8 bit integer |    0x30    |             |
|     unsigned 16 bit integer |    0x31    |             |
|     unsigned 32 bit integer |    0x32    |             |
|     unsigned 64 bit integer |    0x33    |             |
| unsigned system bit integer |    0x34    |             |
