#![allow(warnings)]

use grust::grust;

grust! {
    #![dump = "examples/array_operations/out/dumped.rs", levenshtein = false]
    array MyArray [int; 10]
    array MyMatrix [MyArray; 10]

    const ZEROS: MyArray = [0; 10];

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

    function sort_my_array(to_sort: MyArray) -> MyArray {
        return to_sort.sort(|a: int, b: int| a-b);
    }

    function sum_by_fold(a: int, b: int) -> int {
        let my_array: [int; 5] = [1, 2, 3, a, b];
        let sum: int = my_array.fold(
            0,
            |x: int, y: int| x + y
        );
        return sum;
    }

    function map_two(a1: MyArray, a2: MyArray, f: ((int, int)) -> int) -> MyArray {
        return zip(a1, ZEROS).map(f);
    }

    enum Priority {
        Hight, Medium, Low,
    }

    struct Alarm {
        prio: Priority,
        data: int,
    }

    struct FilteredAlarm {
        alarm: Alarm,
        filtered: bool,
    }

    function alarm_filtering(alarm_list: [Alarm; 100]) -> [FilteredAlarm; 100] {
        return alarm_list.map(|alarm: Alarm| FilteredAlarm {
            alarm: alarm, filtered: alarm.data <= 0,
        }).sort(
            |a1: FilteredAlarm, a2: FilteredAlarm| match (a1.alarm.prio, a2.alarm.prio) {
                (p1, p2) if p1 == p2    => 0,
                (Priority::Hight, _)    => -1,
                (_, Priority::Hight)    => 1,
                (Priority::Medium, _)   => -1,
                (_, Priority::Medium)   => 1,
                _                       => 0
            }
        );
    }

    function levenshtein_horror(variable_with_a_very_long_name: int) -> int {
        return variable_with_a_very_long_name;
    }
}
