mod dependencies;
mod equation;
mod file;
mod stream_expression;
mod unitary_node;

#[derive(PartialEq, Debug)]
pub enum Union<U, V> {
    I1(U),
    I2(V),
}
