use std::time::Duration;

use colored::Colorize;
use futures::StreamExt;
use greact::event::{input_channel, Event};
use tokio::{sync::mpsc::channel, time::sleep};

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
#[tokio::test]
async fn main() {
    // events management
    let (tx_i, i) = input_channel();
    let (tx_o, mut rx_o) = channel::<i64>(1);

    // synchronous state-machine initialisation
    let mut state = MainState::init();

    // input/output management pushed in another thread
    tokio::spawn(async move {
        let mut n = 0;
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

    // asynchronous state-machine launched
    let i = i.push_timeout(Duration::from_millis(100));
    tokio::pin!(i);
    loop {
        let a = i.next().await.expect("should not end");
        println!("{}", format!("input received: {a:?}").red());
        let b = state.step(MainInput { a });
        println!("{}", format!("compute: {b}").blue());
        if let Err(_) = tx_o.send(b).await {
            println!("output receiver dropped");
            return;
        }
    }
}
