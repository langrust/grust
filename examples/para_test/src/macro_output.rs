pub struct Test1Input {
    pub i: i64,
}
pub struct Test1State {
    last_i: i64,
}
impl Test1State {
    pub fn init() -> Test1State {
        Test1State { last_i: 0i64 }
    }
    pub fn step(&mut self, input: Test1Input) -> i64 {
        let (i1, i3, i2) = std::thread::scope(|reserved_grust_thread_scope| {
            let reserved_grust_thread_kid_0 =
                reserved_grust_thread_scope.spawn(|| (input.i - 54i64) * 2i64);
            let reserved_grust_thread_kid_1 = reserved_grust_thread_scope.spawn(|| 7i64 * input.i);
            let reserved_grust_thread_kid_2 =
                reserved_grust_thread_scope.spawn(|| (input.i + 54i64) * 2i64);
            let (i1, i3, i2) = (
                reserved_grust_thread_kid_0
                    .join()
                    .expect("unexpected panic in sub-thread"),
                reserved_grust_thread_kid_1
                    .join()
                    .expect("unexpected panic in sub-thread"),
                reserved_grust_thread_kid_2
                    .join()
                    .expect("unexpected panic in sub-thread"),
            );
            (i1, i3, i2)
        });
        let (i12, i23) = std::thread::scope(|reserved_grust_thread_scope| {
            let reserved_grust_thread_kid_0 = reserved_grust_thread_scope.spawn(|| i1 + i2);
            let reserved_grust_thread_kid_1 = reserved_grust_thread_scope.spawn(|| i2 + i3);
            let (i12, i23) = (
                reserved_grust_thread_kid_0
                    .join()
                    .expect("unexpected panic in sub-thread"),
                reserved_grust_thread_kid_1
                    .join()
                    .expect("unexpected panic in sub-thread"),
            );
            (i12, i23)
        });
        let i123 = (i12 + (2i64 * i3)) + i23;
        let next_o = match input.i {
            0 => {
                let next_o = 1i64 + self.last_i;
                next_o
            }
            7 => {
                let next_o = i123;
                next_o
            }
            _ => {
                let next_o = i12;
                next_o
            }
        };
        self.last_i = input.i;
        next_o
    }
}
