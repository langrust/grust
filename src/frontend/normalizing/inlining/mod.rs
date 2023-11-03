mod dependencies;
mod equation;
mod file;
mod memory;
mod node;
mod stream_expression;
mod unitary_node;

#[derive(PartialEq, Debug, Clone)]
pub enum Union<U, V> {
    I1(U),
    I2(V),
}
