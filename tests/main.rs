use occlusion::{Projections};

#[derive(Projections)]
struct Person {
    #[occlude(Aak)]
    name: String,
    age: u32,
    #[occlude(Aak, Bek)]
    sex: Option<bool>,
}
