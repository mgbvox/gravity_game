use game_macros::{FieldIter, key_name};
use bevy::prelude::KeyCode;

#[derive(FieldIter)]
struct Foo {
    bar: String,
    baz: i8,
}

impl Default for Foo {
    fn default() -> Self {
        Self {
            bar: "Heyo".to_string(),
            baz: 0,
        }
    }
}




#[test]
fn test() {
    let foo = Foo::default();
    let fields = foo.iter_fields();
    assert_eq!(fields.len(), 2);
}

#[test]
fn test_key_name() {
    assert_eq!(key_name!(KeyCode::KeyM), "M");
    println!("{:?}", KeyCode::KeyM);
}
