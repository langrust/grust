#![allow(dead_code)]
pub mod trigo {
    /// Computes the cosine of a number (in radians).
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    #[inline]
    pub fn cos(x: f64) -> f64 {
        x.cos()
    }

    /// Computes the sine of a number (in radians).
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    #[inline]
    pub fn sin(x: f64) -> f64 {
        x.sin()
    }

    /// Computes the tangent of a number (in radians).
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    /// This function currently corresponds to the `tan` from libc on Unix and
    /// Windows. Note that this might change in the future.
    #[inline]
    pub fn tan(x: f64) -> f64 {
        x.tan()
    }

    /// Computes the arccosine of a number. Return value is in radians in
    /// the range [0, pi] or NaN if the number is outside the range
    /// [-1, 1].
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    /// This function currently corresponds to the `acos` from libc on Unix and
    /// Windows. Note that this might change in the future.
    #[inline]
    pub fn acos(x: f64) -> f64 {
        x.acos()
    }

    /// Computes the arcsine of a number. Return value is in radians in
    /// the range [-pi/2, pi/2] or NaN if the number is outside the range
    /// [-1, 1].
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    /// This function currently corresponds to the `asin` from libc on Unix and
    /// Windows. Note that this might change in the future.
    #[inline]
    pub fn asin(x: f64) -> f64 {
        x.asin()
    }

    /// Computes the arctangent of a number. Return value is in radians in the
    /// range [-pi/2, pi/2];
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    /// This function currently corresponds to the `atan` from libc on Unix and
    /// Windows. Note that this might change in the future.
    #[inline]
    pub fn atan(x: f64) -> f64 {
        x.atan()
    }

    /// Computes the four quadrant arctangent of `y` and `x` in radians.
    ///
    /// * `x = 0`, `y = 0`: `0`
    /// * `x >= 0`: `arctan(y/x)` -> `[-pi/2, pi/2]`
    /// * `y >= 0`: `arctan(y/x) + pi` -> `(pi/2, pi]`
    /// * `y < 0`: `arctan(y/x) - pi` -> `(-pi, -pi/2)`
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    /// This function currently corresponds to the `atan2` from libc on Unix
    /// and Windows. Note that this might change in the future.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use grust_std::maths::trigo;
    ///
    /// // Positive angles measured counter-clockwise
    /// // from positive x axis
    /// // -pi/4 radians (45 deg clockwise)
    /// let x1 = 3.0_f64;
    /// let y1 = -3.0_f64;
    ///
    /// // 3pi/4 radians (135 deg counter-clockwise)
    /// let x2 = -3.0_f64;
    /// let y2 = 3.0_f64;
    ///
    /// let abs_difference_1 = (trigo::atan2(y1, x1) - (-std::f64::consts::FRAC_PI_4)).abs();
    /// let abs_difference_2 = (trigo::atan2(y2, x2) - (3.0 * std::f64::consts::FRAC_PI_4)).abs();
    ///
    /// assert!(abs_difference_1 < 1e-10);
    /// assert!(abs_difference_2 < 1e-10);
    /// ```
    #[inline]
    pub fn atan2(y: f64, x: f64) -> f64 {
        y.atan2(x)
    }

    /// Hyperbolic cosine function.
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    /// This function currently corresponds to the `cosh` from libc on Unix
    /// and Windows. Note that this might change in the future.
    #[inline]
    pub fn cosh(x: f64) -> f64 {
        x.cosh()
    }

    /// Hyperbolic sine function.
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    /// This function currently corresponds to the `sinh` from libc on Unix
    /// and Windows. Note that this might change in the future.
    #[inline]
    pub fn sinh(x: f64) -> f64 {
        x.sinh()
    }

    /// Hyperbolic tangent function.
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    /// This function currently corresponds to the `tanh` from libc on Unix
    /// and Windows. Note that this might change in the future.
    #[inline]
    pub fn tanh(x: f64) -> f64 {
        x.tanh()
    }

    /// Inverse hyperbolic cosine function.
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    #[inline]
    pub fn acosh(x: f64) -> f64 {
        x.acosh()
    }

    /// Inverse hyperbolic sine function.
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    #[inline]
    pub fn asinh(x: f64) -> f64 {
        x.asinh()
    }

    /// Inverse hyperbolic tangent function.
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    #[inline]
    pub fn atanh(x: f64) -> f64 {
        x.atanh()
    }
}

pub mod round {
    /// Round function
    /// Returns the nearest integer to `x`. If a value is half-way between two
    /// integers, round away from `0`.
    ///
    /// This function always returns the precise result.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use grust_std::maths::round;
    ///
    /// let f = 3.3_f64;
    /// let g = -3.3_f64;
    /// let h = -3.7_f64;
    /// let i = 3.5_f64;
    /// let j = 4.5_f64;
    ///
    /// assert_eq!(round::round(f), 3);
    /// assert_eq!(round::round(g), -3);
    /// assert_eq!(round::round(h), -4);
    /// assert_eq!(round::round(i), 4);
    /// assert_eq!(round::round(j), 5);
    /// ```
    #[inline]
    pub fn round(x: f64) -> i64 {
        x.round() as i64
    }

    /// Floor function
    /// Returns the largest integer less than or equal to `x`.
    ///
    /// This function always returns the precise result.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use grust_std::maths::round;
    ///
    /// let f = 3.7_f64;
    /// let g = 3.0_f64;
    /// let h = -3.7_f64;
    ///
    /// assert_eq!(round::floor(f), 3);
    /// assert_eq!(round::floor(g), 3);
    /// assert_eq!(round::floor(h), -4);
    /// ```
    #[inline]
    pub fn floor(x: f64) -> i64 {
        x.floor() as i64
    }

    /// Ceiling function
    /// Returns the smallest integer greater than or equal to `x`.
    ///
    /// This function always returns the precise result.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use grust_std::maths::round;
    ///
    /// let f = 3.01_f64;
    /// let g = 4.0_f64;
    ///
    /// assert_eq!(round::ceil(f), 4);
    /// assert_eq!(round::ceil(g), 4);
    /// ```
    #[inline]
    pub fn ceil(x: f64) -> i64 {
        x.ceil() as i64
    }
}

pub mod exponential {
    /// Returns `e^(x)`, (the exponential function).
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    pub fn exp(x: f64) -> f64 {
        x.exp()
    }

    /// Returns the natural logarithm of the number.
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    pub fn ln(x: f64) -> f64 {
        x.ln()
    }

    /// Returns the logarithm of the number with respect to an arbitrary base.
    ///
    /// The result might not be correctly rounded owing to implementation details;
    /// `log2(x)` can produce more accurate results for base 2, and
    /// `log10(x)` can produce more accurate results for base 10.
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    pub fn log(x: f64, n: f64) -> f64 {
        x.log(n)
    }

    /// Returns the base 10 logarithm of the number.
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    pub fn log10(x: f64) -> f64 {
        x.log10()
    }

    /// Returns the base 2 logarithm of the number.
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    pub fn log2(x: f64) -> f64 {
        x.log2()
    }
}

pub mod usuals {
    /// Computes the minimum between 2 integers.
    #[inline]
    pub fn min(x: f64, y: f64) -> f64 {
        if x <= y {
            x
        } else {
            y
        }
    }

    /// Computes the minimun between 2 floats.
    #[inline]
    pub fn min_i(x: i64, y: i64) -> i64 {
        if x <= y {
            x
        } else {
            y
        }
    }

    /// Computes the maximum between 2 floats.
    #[inline]
    pub fn max(x: f64, y: f64) -> f64 {
        if x >= y {
            x
        } else {
            y
        }
    }

    /// Computes the maximum between 2 integers.
    #[inline]
    pub fn max_i(x: i64, y: i64) -> i64 {
        if x >= y {
            x
        } else {
            y
        }
    }

    /// Computes the absolute value of `x` for floats.
    #[inline]
    pub fn abs(x: f64) -> f64 {
        x.abs()
    }

    /// Computes the absolute value of `x` for integers.
    #[inline]
    pub fn abs_i(x: i64) -> i64 {
        if x >= 0 {
            x
        } else {
            -x
        }
    }

    /// Raises a number to a floating point power.
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    #[inline]
    pub fn pow(x: f64, n: f64) -> f64 {
        x.powf(n)
    }

    /// Raises a number to a floating point power.
    ///
    /// # Unspecified precision
    ///
    /// The precision of this function is non-deterministic. This means it varies by platform, Rust version, and
    /// can even differ within the same execution from one invocation to the next.
    #[inline]
    pub fn pow_i(x: i64, n: i64) -> i64 {
        let x = x as f64;
        let n = n as f64;
        x.powf(n) as i64
    }

    /// Returns the square root of a float.
    ///
    /// Panics if `x` is a negative number other than `-0.0`.
    ///
    /// # Precision
    ///
    /// The result of this operation is guaranteed to be the rounded
    /// infinite-precision result. It is specified by IEEE 754 as `squareRoot`
    /// and guaranteed not to change.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use grust_std::maths::usuals;
    ///
    /// let positive = 4.0_f64;
    /// let negative_zero = -0.0_f64;
    ///
    /// assert_eq!(usuals::sqrt(positive), 2.0);
    /// assert!(usuals::sqrt(negative_zero) == negative_zero);
    /// ```
    #[inline]
    pub fn sqrt(x: f64) -> f64 {
        let result = x.sqrt();
        if result.is_nan() {
            panic!("Sqrt function can't use negative values")
        } else {
            result
        }
    }

    /// Returns the square root of an integer.
    ///
    /// Panics if `x` is a negative number other than `-0.0`.
    ///
    /// # Precision
    ///
    /// The result of this operation is guaranteed to be the rounded
    /// infinite-precision result. It is specified by IEEE 754 as `squareRoot`
    /// and guaranteed not to change.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use grust_std::maths::usuals;
    ///
    /// let positive = 4i64;
    /// let negative_zero = -0i64;
    ///
    /// assert_eq!(usuals::sqrt_i(positive), 2.0);
    /// assert!(usuals::sqrt_i(negative_zero) == 0.0);
    /// ```
    #[inline]
    pub fn sqrt_i(x: i64) -> f64 {
        let x = x as f64;
        let result = x.sqrt();
        if result.is_nan() {
            panic!("Sqrt function can't use negative values")
        } else {
            result
        }
    }
}
