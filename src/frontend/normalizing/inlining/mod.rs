mod dependencies;
mod expression;
mod file;
mod memory;
mod node;
mod statement;
mod stream_expression;
mod unitary_node;

#[derive(PartialEq, Debug, Clone)]
pub enum Union<U, V> {
    I1(U),
    I2(V),
}
