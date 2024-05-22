use std::{thread, time::Duration};

use colored::Colorize;
use greact::{
    event::{input_channel, Event},
    stream::pull_stream::PullStream,
};
use tokio::sync::mpsc::channel;

struct MainInput {
    a: Option<i64>,
}
struct MainState {
    mem: i64,
}
impl MainState {
    fn init() -> Self {
        Self { mem: 0 }
    }
    fn step(&mut self, input: MainInput) -> i64 {
        let o = match input.a {
            Some(a) => a + self.mem,
            None => self.mem,
        };
        self.mem = o;
        o
    }
}

/// ```GRRust
/// component main(a: int?) {
///     mem: int = 0 fby o;
/// 	out o: int = when a then a + mem else mem;
/// }
///
/// interface {
///     event int i; // import sdv.adas.fusion.i1;
///     event int o; // import sdv.adas.fusion.o;
///
///     stream int? a = event_sample(i);
///     stream int b = main(a).o;
///
///     o = event_emit(b);
/// }
/// ```
#[test]
fn main() {
    // events management
    let (tx_i, i) = input_channel::<i64>();
    let (tx_o, mut rx_o) = channel::<i64>(1);

    // synchronous state-machine initialisation
    let mut state = MainState::init();

    // input/output management pushed in another thread
    thread::spawn(move || {
        let mut n = 0;
        loop {
            println!("{}", format!("emit: {n}").green());
            if let Err(_) = tx_i.blocking_send(n) {
                println!("input receiver dropped");
                return;
            }
            n += 1;
            thread::sleep(Duration::from_millis(200));
        }
    });
    thread::spawn(move || loop {
        let n = rx_o.blocking_recv().expect("should not end");
        println!("{}", format!("output received: {n}").yellow());
    });

    // synchronous state-machine launched
    let mut i = i.pull();
    thread::sleep(Duration::from_millis(1));
    loop {
        let a = i.pick();
        println!("{}", format!("input sampled: {a:?}").red());
        let b = state.step(MainInput { a });
        println!("{}", format!("compute: {b}").blue());
        if let Err(_) = tx_o.blocking_send(b) {
            println!("output receiver dropped");
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
}
