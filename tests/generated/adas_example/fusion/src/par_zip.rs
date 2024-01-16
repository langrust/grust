//! Parallele zipper module.
//!
//! This module defines the parallele zipper macro.

/// Macro for zipping multiple parallele iterators.
///
/// # Example
/// ```
/// use grfast::par_zip;
/// use rayon::iter::{IndexedParallelIterator, ParallelIterator};
///
/// let iter1 = 0..10;
/// let iter2 = 10..20;
/// let iter3 = 20..30;
/// let iter123 = par_zip!(iter1, iter2, iter3);
/// let result: Vec<_> = iter123.take(2).collect();
/// assert_eq!(result, [(0, 10, 20), (1, 11, 21)]);
/// ```
#[macro_export]
macro_rules! par_zip {
    // @closure creates a tuple-flattening closure for .map() call. usage:
    // @closure partial_pattern => partial_tuple , rest , of , iterators
    // eg. par_zip!( @closure ((a, b), c) => (a, b, c) , dd , ee )
    ( @closure $p:pat => $tup:expr ) => {
        |$p| $tup
    };

    // The "b" identifier is a different identifier on each recursion level thanks to hygiene.
    ( @closure $p:pat => ( $($tup:tt)* ) , $_iter:expr $( , $tail:expr )* ) => {
        par_zip!(@closure ($p, b) => ( $($tup)*, b ) $( , $tail )*)
    };

    // unary
    ($first:expr) => {
        rayon::iter::IntoParallelIterator::into_par_iter($first)
    };

    // n-ary where n > 2
    ( $first:expr , $( $rest:expr ),* ) => {
        {
            let temp_zip = par_zip!($first);
            $(
                let temp_zip = rayon::iter::IndexedParallelIterator::zip(temp_zip, $rest);
            )*
            rayon::iter::ParallelIterator::map(
                temp_zip,
                par_zip!(@closure a => (a) $( , $rest )*)
            )
        }
    };
}

#[cfg(test)]
mod par_zip {
    use rayon::iter::{IndexedParallelIterator, ParallelIterator};

    #[test]
    fn test_par_zip() {
        let iter1 = 0..10;
        let iter2 = 10..20;
        let iter3 = 20..30;
        let iter123 = par_zip!(iter1, iter2, iter3);
        let result: Vec<_> = iter123.take(2).collect();
        assert_eq!(result, [(0, 10, 20), (1, 11, 21)]);
    }
}
