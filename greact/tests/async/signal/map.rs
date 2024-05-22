use std::time::Duration;

use colored::Colorize;
use futures::StreamExt;
use greact::signal::{input_channel, map, Signal};
use tokio::{sync::mpsc::channel, time::sleep};

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
///     signal int e = map(i, |x| x * 2);
///
///     stream int a = signal_sample(e);
///     stream int b = main(a).o;
///
///     o = signal_emit(b);
/// }
/// ```
#[tokio::test]
async fn main() {
    // signals management
    let (tx_i, i) = input_channel(0);
    let (tx_o, mut rx_o) = channel::<i64>(1);
    let e = map(i, |x: i64| x * 2); // construct the MapSignal structure

    // synchronous state-machine initialisation
    let mut state = MainState::init();

    // input/output management pushed in another thread
    tokio::spawn(async move {
        let mut n = 0;
        sleep(Duration::from_millis(1)).await;
        loop {
            println!("{}", format!("emit: {n}").green());
            if let Err(_) = tx_i.send(n).await {
                println!("input receiver dropped");
                return;
            }
            n += 1;
            sleep(Duration::from_millis(200)).await;
        }
    });
    tokio::spawn(async move {
        loop {
            let n = rx_o.recv().await.expect("should not end");
            println!("{}", format!("output received: {n}").yellow());
        }
    });

    // synchronous state-machine launched
    let e = e.push();
    tokio::pin!(e);
    loop {
        let a = e.next().await.expect("should not end");
        println!("{}", format!("input received: {a:?}").red());
        let b = state.step(MainInput { a });
        println!("{}", format!("compute: {b}").blue());
        if let Err(_) = tx_o.send(b).await {
            println!("output receiver dropped");
            return;
        }
    }
}
