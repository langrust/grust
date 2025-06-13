mod scan_sample {
    use grust::grust;

    grust! {
        #![mode = demo, dump = "examples/scan_sample/out/dumped.rs"]
        import signal temperature: float;
        import event pedestrian: float;

        export signal scanned_temperature: float;
        export event sampled_pedestrian: float;

        service scan_sample @ [10, 3000] {
            scanned_temperature = scan(temperature, 100);
            sampled_pedestrian = sample(pedestrian, 250);
        }
    }
}

use futures::{Stream, StreamExt};
use lazy_static::lazy_static;
use priority_stream::prio_stream;
use scan_sample::runtime::{Runtime, RuntimeInit, RuntimeInput, RuntimeOutput, RuntimeTimer};
use std::time::Instant;
use timer_stream::timer_stream;

lazy_static! {
    /// Initial instant.
    static ref INIT : Instant = Instant::now();
}

pub fn run_scan_sample(
    import_stream: impl Stream<Item = RuntimeInput> + Send + 'static,
) -> impl Stream<Item = RuntimeOutput> {
    let (timers_sink, timers_stream) = futures::channel::mpsc::channel(4);
    let (output_sink, output_stream) = futures::channel::mpsc::channel(4);

    let timers_stream = timer_stream::<_, _, 4>(timers_stream)
        .map(|(timer, deadline): (RuntimeTimer, Instant)| RuntimeInput::Timer(timer, deadline));
    let input_stream = prio_stream::<_, _, 3>(
        futures::stream::select(import_stream, timers_stream),
        RuntimeInput::order,
    );

    let scan_sample_service = Runtime::new(output_sink, timers_sink);
    tokio::spawn(scan_sample_service.run_loop(
        *INIT,
        input_stream,
        RuntimeInit { temperature: 10.0 },
    ));

    output_stream
}

#[cfg(test)]
mod sample_scan {
    use super::*;
    use std::time::{Duration, Instant};
    use tokio::time::sleep;

    #[tokio::test]
    async fn should_scan_and_sample_at_right_time() {
        let import_stream = futures::stream::once(async {
            sleep(Duration::from_millis(10)).await;
            RuntimeInput::Pedestrian(100., Instant::now())
        })
        .chain(futures::stream::once(async {
            sleep(Duration::from_millis(110)).await;
            RuntimeInput::Temperature(12., Instant::now())
        }))
        .chain(futures::stream::once(async {
            sleep(Duration::from_millis(20)).await;
            RuntimeInput::Temperature(13., Instant::now())
        }))
        .chain(futures::stream::once(async {
            sleep(Duration::from_millis(90)).await;
            RuntimeInput::Temperature(15., Instant::now())
        }))
        .chain(futures::stream::once(async {
            sleep(Duration::from_millis(50)).await;
            RuntimeInput::Pedestrian(200., Instant::now())
        }))
        .chain(futures::stream::once(async {
            sleep(Duration::from_millis(500)).await;
            RuntimeInput::Pedestrian(300., Instant::now())
        }))
        .chain(futures::stream::once(async {
            sleep(Duration::from_millis(10)).await;
            RuntimeInput::Pedestrian(400., Instant::now())
        }));
        let mut output_stream = run_scan_sample(import_stream);

        assert_eq!(
            output_stream.next().await,
            Some(RuntimeOutput::ScannedTemperature(10.0, *INIT))
        );
        assert_eq!(
            output_stream.next().await,
            Some(RuntimeOutput::ScannedTemperature(
                13.0,
                *INIT + Duration::from_millis(200)
            ))
        );
        assert_eq!(
            output_stream.next().await,
            Some(RuntimeOutput::SampledPedestrian(
                100.0,
                *INIT + Duration::from_millis(250)
            ))
        );
        assert_eq!(
            output_stream.next().await,
            Some(RuntimeOutput::ScannedTemperature(
                15.0,
                *INIT + Duration::from_millis(300)
            ))
        );
        assert_eq!(
            output_stream.next().await,
            Some(RuntimeOutput::SampledPedestrian(
                200.0,
                *INIT + Duration::from_millis(500)
            ))
        );
        assert_eq!(
            output_stream.next().await,
            Some(RuntimeOutput::SampledPedestrian(
                400.0,
                *INIT + Duration::from_millis(1000)
            ))
        );
        assert_eq!(
            output_stream.next().await,
            Some(RuntimeOutput::ScannedTemperature(
                15.0,
                *INIT + Duration::from_millis(4000)
            ))
        );
        assert_eq!(
            output_stream.next().await,
            Some(RuntimeOutput::ScannedTemperature(
                15.0,
                *INIT + Duration::from_millis(7000)
            ))
        );
    }
}
