// tests/integration_tests.rs
use serde_int_tag::IntTag;
use std::collections::BTreeMap;
use std::io::Cursor; // Replace with your crate name // Changed from HashMap

#[derive(IntTag, Default, PartialEq, Debug)]
struct BasicStruct {
    #[tag(1)]
    name: String,
    #[tag(2)]
    age: u32,
    #[tag(3)]
    active: bool,
}

#[derive(IntTag, Default, PartialEq, Debug)]
struct OptionalStruct {
    #[tag(1)]
    name: Option<String>,
    #[tag(2)]
    age: Option<u32>,
}

#[derive(IntTag, Default, PartialEq, Debug)]
struct NestedStruct {
    #[tag(1)]
    id: u32,
    #[tag(2)]
    details: BasicStruct,
}

#[derive(IntTag, Default, PartialEq, Debug)]
struct MapStruct {
    #[tag(1)]
    scores: BTreeMap<u32, String>, // Changed from HashMap
}

#[test]
fn test_basic_struct() {
    let original = BasicStruct {
        name: "test".to_string(),
        age: 42,
        active: true,
    };

    let mut buffer = Vec::new();
    ciborium::ser::into_writer(&original, &mut buffer).unwrap();

    let expected = vec![
        0xA3, // map(3)
        0x01, // unsigned(1)
        0x64, // text(4)
        0x74, 0x65, 0x73, 0x74, // "test"
        0x02, // unsigned(2)
        0x18, 0x2A, // unsigned(42)
        0x03, // unsigned(3)
        0xF5, // true
    ];
    assert_eq!(buffer, expected);

    let deserialized: BasicStruct = ciborium::de::from_reader(Cursor::new(&buffer)).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_optional_struct_all_some() {
    let original = OptionalStruct {
        name: Some("Alice".to_string()),
        age: Some(25),
    };

    let mut buffer = Vec::new();
    ciborium::ser::into_writer(&original, &mut buffer).unwrap();

    let expected = vec![
        0xA2, // map(2)
        0x01, // unsigned(1)
        0x65, // text(5)
        0x41, 0x6C, 0x69, 0x63, 0x65, // "Alice"
        0x02, // unsigned(2)
        0x18, 0x19, // unsigned(25) - minimal encoding
    ];
    assert_eq!(buffer, expected);

    let deserialized: OptionalStruct = ciborium::de::from_reader(Cursor::new(&buffer)).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_optional_struct_all_none() {
    let original = OptionalStruct {
        name: None,
        age: None,
    };

    let mut buffer = Vec::new();
    ciborium::ser::into_writer(&original, &mut buffer).unwrap();

    let expected = vec![0xA0]; // empty map
    assert_eq!(buffer, expected);

    let deserialized: OptionalStruct = ciborium::de::from_reader(Cursor::new(&buffer)).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_nested_struct() {
    let original = NestedStruct {
        id: 123,
        details: BasicStruct {
            name: "Bob".to_string(),
            age: 30,
            active: false,
        },
    };

    let mut buffer = Vec::new();
    ciborium::ser::into_writer(&original, &mut buffer).unwrap();

    let expected = vec![
        0xA2, // map(2)
        0x01, // unsigned(1)
        0x18, 0x7B, // unsigned(123)
        0x02, // unsigned(2)
        0xA3, // map(3)
        0x01, // unsigned(1)
        0x63, // text(3)
        0x42, 0x6F, 0x62, // "Bob"
        0x02, // unsigned(2)
        0x18, 0x1E, // unsigned(30)
        0x03, // unsigned(3)
        0xF4, // false
    ];
    assert_eq!(buffer, expected);

    let deserialized: NestedStruct = ciborium::de::from_reader(Cursor::new(&buffer)).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_map_struct() {
    let mut scores = BTreeMap::new(); // Changed from HashMap
    scores.insert(1, "one".to_string());
    scores.insert(2, "two".to_string());
    let original = MapStruct { scores };

    let mut buffer = Vec::new();
    ciborium::ser::into_writer(&original, &mut buffer).unwrap();

    let expected = vec![
        0xA1, // map(1)
        0x01, // unsigned(1)
        0xA2, // map(2)
        0x01, // unsigned(1)
        0x63, // text(3)
        0x6F, 0x6E, 0x65, // "one"
        0x02, // unsigned(2)
        0x63, // text(3)
        0x74, 0x77, 0x6F, // "two"
    ];
    assert_eq!(buffer, expected);

    let deserialized: MapStruct = ciborium::de::from_reader(Cursor::new(&buffer)).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_missing_tags_deserialization() {
    let partial_data = vec![
        0xA1, // map(1)
        0x01, // unsigned(1)
        0x64, // text(4)
        0x74, 0x65, 0x73, 0x74, // "test"
    ];

    let deserialized: BasicStruct = ciborium::de::from_reader(Cursor::new(&partial_data)).unwrap();
    let expected = BasicStruct {
        name: "test".to_string(),
        age: 0,
        active: false,
    };
    assert_eq!(deserialized, expected);
}

#[test]
fn test_extra_tags_deserialization() {
    let extra_data = vec![
        0xA4, // map(4)
        0x01, // unsigned(1)
        0x64, // text(4)
        0x74, 0x65, 0x73, 0x74, // "test"
        0x02, // unsigned(2)
        0x18, 0x2A, // unsigned(42)
        0x03, // unsigned(3)
        0xF5, // true
        0x04, // unsigned(4) - unknown tag
        0x18, 0xFF, // unsigned(255)
    ];

    let deserialized: BasicStruct = ciborium::de::from_reader(Cursor::new(&extra_data)).unwrap();
    let expected = BasicStruct {
        name: "test".to_string(),
        age: 42,
        active: true,
    };
    assert_eq!(deserialized, expected);
}

#[test]
fn test_empty_map_deserialization() {
    let empty_data = vec![0xA0]; // empty map

    let deserialized: BasicStruct = ciborium::de::from_reader(Cursor::new(&empty_data)).unwrap();
    let expected = BasicStruct::default();
    assert_eq!(deserialized, expected);
}
