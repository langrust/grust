pub struct UsingPhantomInput {
    pub j: i64,
}
pub struct UsingPhantomState {}
impl UsingPhantomState {
    pub fn init() -> UsingPhantomState {
        UsingPhantomState {}
    }
    pub fn step(&mut self, input: UsingPhantomInput) -> i64 {
        let phantom = input.j;
        phantom
    }
}
