pub type MyArray = [i64; 10usize];
pub type MyMatrix = [[i64; 10usize]; 10usize];
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum Priority {
    #[default]
    Hight,
    Medium,
    Low,
}
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct Alarm {
    pub prio: Priority,
    pub data: i64,
}
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct FilteredAlarm {
    pub alarm: Alarm,
    pub filtered: bool,
}
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
pub fn sort_my_array(to_sort: [i64; 10usize]) -> [i64; 10usize] {
    {
        let mut grust_reserved_sort = to_sort.clone();
        grust_reserved_sort.sort_by(|a, b| {
            let cmp = |a: i64, b: i64| -> i64 { a - b }(*a, *b);
            if cmp < 0 {
                std::cmp::Ordering::Less
            } else if 0 < cmp {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });
        grust_reserved_sort
    }
}
pub fn sum_by_fold(a: i64, b: i64) -> i64 {
    let my_array = [1i64, 2i64, 3i64, a, b];
    let sum = my_array
        .into_iter()
        .fold(0i64, |x: i64, y: i64| -> i64 { x + y });
    sum
}
pub fn map_two(
    a1: [i64; 10usize],
    a2: [i64; 10usize],
    f: impl Fn((i64, i64)) -> i64,
) -> [i64; 10usize] {
    std::array::from_fn(|n| (a1[n], a2[n])).map(f)
}
pub fn alarm_filtering(alarm_list: [Alarm; 100usize]) -> [FilteredAlarm; 100usize] {
    {
        let mut grust_reserved_sort = alarm_list
            .map(|alarm: Alarm| -> FilteredAlarm {
                FilteredAlarm {
                    alarm: alarm,
                    filtered: alarm.data <= 0i64,
                }
            })
            .clone();
        grust_reserved_sort.sort_by(|a, b| {
            let cmp = |a1: FilteredAlarm, a2: FilteredAlarm| -> i64 {
                match (a1.alarm.prio, a2.alarm.prio) {
                    (p1, p2) if p1 == p2 => 0i64,
                    (Priority::Hight, _) => -1i64,
                    (_, Priority::Hight) => 1i64,
                    (Priority::Medium, _) => -1i64,
                    (_, Priority::Medium) => 1i64,
                    _ => 0i64,
                }
            }(*a, *b);
            if cmp < 0 {
                std::cmp::Ordering::Less
            } else if 0 < cmp {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });
        grust_reserved_sort
    }
}
