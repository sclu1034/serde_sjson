use serde_sjson::to_string;

#[test]
fn serialize_null() {
    #[derive(serde::Serialize)]
    struct Value {
        value: (),
    }

    let value = Value { value: () };
    let expected = String::from("value = null\n");
    let actual = to_string(&value).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn serialize_u64() {
    #[derive(serde::Serialize)]
    struct Value {
        value: u64,
    }

    let tests = [u64::MIN, 4, u64::MAX];
    for value in tests {
        let expected = format!("value = {value}\n");
        let value = Value { value };
        let actual = to_string(&value).unwrap();
        assert_eq!(actual, expected);
    }
}

#[test]
fn serialize_i64() {
    #[derive(serde::Serialize)]
    struct Value {
        value: i64,
    }

    let tests = [i64::MIN, -6, 0, 4, i64::MAX];
    for value in tests {
        let expected = format!("value = {value}\n");
        let value = Value { value };
        let actual = to_string(&value).unwrap();
        assert_eq!(actual, expected);
    }
}

#[test]
fn serialize_f64() {
    #[derive(serde::Serialize)]
    struct Value {
        value: f64,
    }

    let tests = [
        f64::MIN,
        -12.3456,
        4.3,
        f64::MAX,
        f64::EPSILON,
        std::f64::consts::PI,
    ];
    for value in tests {
        let expected = format!("value = {value}\n");
        let value = Value { value };
        let actual = to_string(&value).unwrap();
        assert_eq!(actual, expected);
    }
}

#[test]
fn serialize_non_representable_floats() {
    #[derive(serde::Serialize)]
    struct Value64 {
        value: f64,
    }

    #[derive(serde::Serialize)]
    struct Value32 {
        value: f32,
    }

    let tests = [std::f64::NAN, std::f64::INFINITY, std::f64::NEG_INFINITY];
    let expected = String::from("value = null\n");
    for value in tests {
        let value = Value64 { value };
        let actual = to_string(&value).unwrap();
        assert_eq!(actual, expected);
    }
    let tests = [std::f32::NAN, std::f32::INFINITY, std::f32::NEG_INFINITY];
    for value in tests {
        let value = Value32 { value };
        let actual = to_string(&value).unwrap();
        assert_eq!(actual, expected);
    }
}

#[test]
fn serialize_bool() {
    #[derive(serde::Serialize)]
    struct Value {
        value: bool,
    }

    let tests = [true, false];
    for value in tests {
        let expected = format!("value = {value}\n");
        let value = Value { value };
        let actual = to_string(&value).unwrap();
        assert_eq!(actual, expected);
    }
}

#[test]
fn serialize_string() {
    #[derive(serde::Serialize)]
    struct Value {
        value: String,
    }

    let tests = [
        ("", "\"\""),
        ("foo", "foo"),
        ("foo bar", "\"foo bar\""),
        ("foo\nbar", "\"foo\\nbar\""),
        ("foo\r\nbar", "\"foo\\r\\nbar\""),
        ("foo\tbar", "\"foo\\tbar\""),
        ("foo/bar", "foo/bar"),
        ("foo\\bar", "\"foo\\\\bar\""),
        // Regression test for #7.
        ("scripts/mods/test\\new", "\"scripts/mods/test\\\\new\""),
        // Regression test for #8.
        (
            "+002023-03-03T16:42:33.944311860Z",
            "\"+002023-03-03T16:42:33.944311860Z\"",
        ),
    ];
    for (value, expected) in tests {
        let expected = format!("value = {expected}\n");
        let value = Value {
            value: value.to_string(),
        };
        let actual = to_string(&value).unwrap();
        assert_eq!(actual, expected);
    }
}

#[test]
fn serialize_char() {
    #[derive(serde::Serialize)]
    struct Value {
        value: String,
    }

    let tests = [
        (' ', "\" \""),
        ('f', "f"),
        ('\n', "\"\\n\""),
        ('\t', "\"\\t\""),
        ('\r', "\"\\r\""),
        ('\\', "\"\\\\\""),
        ('/', "/"),
        ('\"', "\"\\\"\""),
        ('\'', "\"'\""),
    ];
    for (value, expected) in tests {
        let expected = format!("value = {expected}\n");
        let value = Value {
            value: value.to_string(),
        };
        let actual = to_string(&value).unwrap();
        assert_eq!(actual, expected);
    }
}

#[test]
fn serialize_vec_of_strings() {
    #[derive(serde::Serialize)]
    struct Value {
        value: Vec<&'static str>,
    }

    let value = Value {
        value: vec!["foo", "foo bar", "foo123"],
    };
    let expected = String::from(
        "\
value = [
  foo
  \"foo bar\"
  foo123
]
",
    );

    let actual = to_string(&value).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn serialize_nested_vec_of_numbers() {
    #[derive(serde::Serialize)]
    struct Value {
        value: Vec<u64>,
    }

    let value = Value {
        value: vec![0, 1, 1234],
    };
    let actual = to_string(&value).unwrap();
    let expected = String::from(
        "\
value = [
  0
  1
  1234
]
",
    );
    assert_eq!(actual, expected);
}

#[test]
fn serialize_flat_struct() {
    #[derive(serde::Serialize)]
    #[serde(tag = "animal")]
    struct Cat {
        name: String,
        lives: usize,
    }

    let luna = Cat {
        name: String::from("Luna"),
        lives: 9,
    };
    let actual = to_string(&luna).unwrap();
    let expected = String::from("animal = Cat\nname = Luna\nlives = 9\n");

    assert_eq!(actual, expected);
}

#[test]
fn serialize_list_of_structs() {
    #[derive(serde::Serialize)]
    struct Dog {
        name: String,
    }

    let buddy = Dog {
        name: String::from("Buddy"),
    };

    let lotta = Dog {
        name: String::from("Lotta"),
    };

    #[derive(serde::Serialize)]
    struct Value {
        value: Vec<Dog>,
    }
    {
        let actual = to_string(&Value {
            value: vec![buddy, lotta],
        })
        .unwrap();
        let expected = String::from(
            "\
value = [
  {
    name = Buddy
  }
  {
    name = Lotta
  }
]
",
        );

        assert_eq!(actual, expected);
    }
}

#[test]
fn serialize_nested_struct() {
    #[derive(serde::Serialize)]
    struct Dog {
        name: String,
    }

    #[derive(serde::Serialize)]
    struct DogHouse<'a> {
        dog: &'a Dog,
        size: usize,
    }

    let buddy = Dog {
        name: String::from("Buddy"),
    };

    let value = DogHouse {
        dog: &buddy,
        size: 50,
    };
    let actual = to_string(&value).unwrap();
    let expected = String::from(
        "\
dog = {
  name = Buddy
}
size = 50
",
    );

    assert_eq!(actual, expected);
}

#[test]
fn serialize_deeply_nested_struct() {
    #[derive(serde::Serialize)]
    struct Dog {
        name: String,
    }

    #[derive(serde::Serialize)]
    struct Cat {
        name: String,
    }

    #[derive(serde::Serialize)]
    struct DogHouse {
        dog: Dog,
        size: usize,
    }

    #[derive(serde::Serialize)]
    struct Garden {
        dog_house: DogHouse,
        cat: Cat,
    }

    let value = Garden {
        dog_house: DogHouse {
            dog: Dog {
                name: String::from("Buddy"),
            },
            size: 50,
        },
        cat: Cat {
            name: String::from("Luna"),
        },
    };
    let actual = to_string(&value).unwrap();
    let expected = String::from(
        "\
dog_house = {
  dog = {
    name = Buddy
  }
  size = 50
}
cat = {
  name = Luna
}
",
    );

    assert_eq!(actual, expected);
}

#[test]
fn serialize_struct_with_vec_of_structs() {
    #[derive(serde::Serialize)]
    struct Dog {
        name: String,
    }

    let buddy = Dog {
        name: String::from("Buddy"),
    };

    let lotta = Dog {
        name: String::from("Lotta"),
    };

    #[derive(serde::Serialize)]
    struct DogHotel<'a> {
        dogs: Vec<&'a Dog>,
    }

    let value = DogHotel {
        dogs: vec![&buddy, &lotta],
    };
    let actual = to_string(&value).unwrap();
    let expected = String::from(
        "\
dogs = [
  {
    name = Buddy
  }
  {
    name = Lotta
  }
]
",
    );

    assert_eq!(actual, expected);
}

#[test]
fn serialize_enum_variant() {
    #[derive(serde::Serialize)]
    enum Color {
        Red,
    }

    #[derive(serde::Serialize)]
    struct Value {
        value: Color,
    }

    let value = Value { value: Color::Red };
    assert_eq!(to_string(&value).unwrap(), String::from("value = Red\n"));
}

#[test]
fn serialize_enum_newtype_variant() {
    #[derive(serde::Serialize)]
    enum Variant {
        Int(i64),
    }

    #[derive(serde::Serialize)]
    struct Value {
        value: Variant,
    }

    let value = Value {
        value: Variant::Int(13),
    };
    assert_eq!(
        to_string(&value).unwrap(),
        String::from("value = { Int = 13 }\n")
    );
}

#[test]
fn serialize_enum_tagged_variant() {
    #[derive(serde::Serialize)]
    #[serde(tag = "variant", content = "value")]
    enum TaggedVariant {
        Int(i64),
    }

    #[derive(serde::Serialize)]
    struct Value {
        value: TaggedVariant,
    }

    let value = Value {
        value: TaggedVariant::Int(13),
    };
    assert_eq!(
        to_string(&value).unwrap(),
        String::from(
            "\
value = {
  variant = Int
  value = 13
}
"
        )
    );
}

#[test]
fn serialize_tuple_single() {
    #[derive(serde::Serialize)]
    struct Value {
        value: ((),),
    }

    let value = Value { value: ((),) };
    assert_eq!(
        to_string(&value).unwrap(),
        String::from("value = [\n  null\n]\n")
    );
}

#[test]
fn serialize_tuple_multiple_unit() {
    #[derive(serde::Serialize)]
    struct Value {
        value: ((), ()),
    }

    let value = Value { value: ((), ()) };
    assert_eq!(
        to_string(&value).unwrap(),
        String::from("value = [\n  null\n  null\n]\n")
    );
}

#[test]
fn serialize_tuple_multiple_value() {
    #[derive(serde::Serialize)]
    struct Value {
        value: (&'static str, bool),
    }

    let value = Value {
        value: ("foo", false),
    };
    assert_eq!(
        to_string(&value).unwrap(),
        String::from("value = [\n  foo\n  false\n]\n")
    );
}

#[test]
fn serialize_option_unit() {
    #[derive(serde::Serialize)]
    struct Value {
        value: Option<()>,
    }

    let value = Value { value: None };
    assert_eq!(to_string(&value).unwrap(), String::from("value = null\n"));

    let value = Value { value: Some(()) };
    assert_eq!(to_string(&value).unwrap(), String::from("value = null\n"));
}

#[test]
fn serialize_option_number() {
    #[derive(serde::Serialize)]
    struct Value {
        value: Option<u64>,
    }

    let value = Value { value: None };
    assert_eq!(to_string(&value).unwrap(), String::from("value = null\n"));

    let value = Value { value: Some(1234) };
    assert_eq!(to_string(&value).unwrap(), String::from("value = 1234\n"));
}

#[test]
fn serialize_option_string() {
    #[derive(serde::Serialize)]
    struct Value {
        value: Option<String>,
    }

    let value = Value { value: None };
    assert_eq!(to_string(&value).unwrap(), String::from("value = null\n"));

    let value = Value {
        value: Some(String::from("foo")),
    };
    assert_eq!(to_string(&value).unwrap(), String::from("value = foo\n"));

    let value = Value {
        value: Some(String::from("foo bar")),
    };
    assert_eq!(
        to_string(&value).unwrap(),
        String::from("value = \"foo bar\"\n")
    );
}
