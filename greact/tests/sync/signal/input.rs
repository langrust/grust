use colored::Colorize;
use greact::{
    signal::{input_channel, Signal},
    stream::pull_stream::PullStream,
};
use std::{thread, time::Duration};
use tokio::sync::mpsc::channel;

struct MainInput {
    a: i64,
}
struct MainState {
    mem: i64,
}
impl MainState {
    fn init() -> Self {
        Self { mem: 0 }
    }
    fn step(&mut self, input: MainInput) -> i64 {
        let o = input.a + self.mem;
        self.mem = o;
        o
    }
}

/// ```GRRust
/// component main(a: int) {
///     mem: int = 0 fby o;
/// 	out o: int = a + mem;
/// }
///
/// interface {
///     signal int i; // import sdv.adas.fusion.i1;
///     signal int o; // import sdv.adas.fusion.o;
///
///     stream int a = signal_sample(i);
///     stream int b = main(a).o;
///
///     o = signal_emit(b);
/// }
/// ```
#[test]
fn main() -> Result<(), String> {
    // signals management
    let (tx_i, i) = input_channel::<i64>(0);
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
    let run = crate::Run::new();
    loop {
        if run.should_stop() {
            break;
        }
        let a = i.pick();
        println!("{}", format!("input sampled: {a:?}").red());
        let b = state.step(MainInput { a });
        println!("{}", format!("compute: {b}").blue());
        if let Err(e) = tx_o.blocking_send(b) {
            return Err(format!("output receiver dropped ({e})"));
        }
        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
