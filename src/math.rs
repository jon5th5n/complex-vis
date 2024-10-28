use std::ops::{Add, Div, Mul, Range, Sub};

pub fn lerp<T>(x: T, from: &Range<T>, to: &Range<T>) -> T
where
    T: Copy,
    T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>,
{
    let from_len = from.end - from.start;
    let to_len = to.end - to.start;

    let to_from_ratio = to_len / from_len;

    let res = to.start + to_from_ratio * (x - from.start);
    res
}
