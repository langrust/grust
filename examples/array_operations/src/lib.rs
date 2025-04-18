#![allow(warnings)]

use grust::grust;

grust! {
    #![dump = "examples/array_operations/out/dumped.rs"]
    array MyArray [int; 10]
    array MyMatrix [MyArray; 10]

    function init(x: int) -> MyArray {
        return [x; 10];
    }

    function first(array: MyArray) -> int {
        return array[0];
    }

    function init_matrix(x: int) -> MyMatrix {
        return [init(x); 10];
    }

    function first_matrix(matrix: MyMatrix) -> int {
        return matrix[0][0];
    }
}
