pub type MyArray = [i64; 10usize];
pub type MyMatrix = [[i64; 10usize]; 10usize];
pub fn init(x: i64) -> [i64; 10usize] {
    [x, x, x, x, x, x, x, x, x, x]
}
pub fn first(array: [i64; 10usize]) -> i64 {
    array[0]
}
pub fn init_matrix(x: i64) -> [[i64; 10usize]; 10usize] {
    [
        init(x),
        init(x),
        init(x),
        init(x),
        init(x),
        init(x),
        init(x),
        init(x),
        init(x),
        init(x),
    ]
}
pub fn first_matrix(matrix: [[i64; 10usize]; 10usize]) -> i64 {
    matrix[0][0]
}
