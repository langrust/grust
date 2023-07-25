mod equation;
mod file;
mod unitary_node;
mod stream_expression;

#[derive(PartialEq, Debug)]
pub enum Union<U, V> {
    I1(U),
    I2(V),
}
