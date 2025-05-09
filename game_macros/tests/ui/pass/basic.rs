use game_macros::FieldIter;

#[derive(FieldIter)]
struct Foo {
    bar: i8,
    baz: String,
}

fn main() {
    let foo = Foo {
        bar: 0,
        baz: "foo".to_string(),
    };
    for field in foo.iter_fields() {
        println!("{}", field);
    }
}