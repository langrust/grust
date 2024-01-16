pub fn fibonacci(n: i64) -> i64 {
    let res = if n <= 0i64 {
        0i64
    } else {
        if n <= 1i64 {
            1i64
        } else {
            fibonacci(n - 1i64) + fibonacci(n - 2i64)
        }
    };
    res
}
pub fn factorial(n: i64) -> i64 {
    let res = if n <= 1i64 {
        1i64
    } else if n > 16i64 {
        factorial(16i64)
    } else {
        n * factorial(n - 1i64)
    };
    res
}

#[cfg(test)]
mod factorial {
    use crate::functions::factorial;

    #[test]
    fn should_compute_factorial_0() {
        assert_eq!(factorial(0), 1)
    }

    #[test]
    fn should_compute_factorial_1() {
        assert_eq!(factorial(1), 1)
    }

    #[test]
    fn should_compute_factorial_16() {
        assert_eq!(factorial(16), 20922789888000)
    }
}
