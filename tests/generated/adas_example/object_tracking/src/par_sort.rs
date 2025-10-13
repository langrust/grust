use rayon::slice::ParallelSliceMut;

pub fn par_sort<A, T>(mut array: A) -> A
where
    A: AsMut<[T]>,
    T: Ord + Send,
{
    let slice = array.as_mut();
    slice.par_sort();

    array
}
