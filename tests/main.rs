use narrow_macro::{Projection};

#[derive(Projection)]
struct Person {
    #[occlude]
    name: String,
    age: u32,
    sex: Option<bool>,
}
