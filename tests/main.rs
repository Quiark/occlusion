use occlusion::{Projections};

#[derive(Projections, Default)]
struct Person {
    #[occlude(Aak)]
    name: String,
    age: u32,
    #[occlude(Aak, Bek)]
    sex: Option<bool>,
}
