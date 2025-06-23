#![allow(warnings)]

use grust::grust;

grust! {
    #![dump = "examples/binop_unop/out/dumped.rs", levenshtein = false]

    function add_mult(a: int, b: int, c: int) -> int {
        return (a + b) * c;
    }
    function add_mod(a: float, b: float, c: float) -> float {
        return (a + b) % c;
    }
    function sub_div(a: int, b: int, c: int) -> int {
        return (a - b) / c;
    }
    function sub_eq(a: float, b: float, c: float) -> bool {
        return (a - b) == c;
    }
    function add_diff(a: int, b: int, c: int) -> bool {
        return (a + b) != c;
    }
    function sub_ge(a: float, b: float, c: float) -> bool {
        return (a - b) >= c;
    }
    function add_gt(a: int, b: int, c: int) -> bool {
        return (a + b) > c;
    }
    function sub_le(a: float, b: float, c: float) -> bool {
        return (a - b) <= c;
    }
    function add_lt_neg(a: int, b: int, c: int) -> bool {
        return (a + b) < -c;
    }
    function le_or_gt(a: float, b: float, c: float) -> bool {
        return (a - b) <= c || (a - b) > c;
    }
    function le_and_ge_not(a: int, b: int, c: int) -> bool {
        return (a - b) <= c && !((a - b) > c);
    }
}
