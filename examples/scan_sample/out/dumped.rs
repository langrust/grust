pub mod runtime {
    use super::*;
    use futures::{sink::SinkExt, stream::StreamExt};
    pub enum RuntimeInput {
        Temperature(f64, std::time::Instant),
        Pedestrian(f64, std::time::Instant),
        Timer(T, std::time::Instant),
    }
    use RuntimeInput as I;
    impl priority_stream::Reset for RuntimeInput {
        fn do_reset(&self) -> bool {
            match self {
                I::Timer(timer, _) => timer_stream::Timing::do_reset(timer),
                _ => false,
            }
        }
    }
    impl PartialEq for RuntimeInput {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (I::Temperature(this, _), I::Temperature(other, _)) => this.eq(other),
                (I::Pedestrian(this, _), I::Pedestrian(other, _)) => this.eq(other),
                (I::Timer(this, _), I::Timer(other, _)) => this.eq(other),
                _ => false,
            }
        }
    }
    impl RuntimeInput {
        pub fn get_instant(&self) -> std::time::Instant {
            match self {
                I::Temperature(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::Pedestrian(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::Timer(_, _grust_reserved_instant) => *_grust_reserved_instant,
            }
        }
        pub fn order(v1: &Self, v2: &Self) -> std::cmp::Ordering {
            v1.get_instant().cmp(&v2.get_instant())
        }
    }
    #[derive(Debug, PartialEq)]
    pub enum RuntimeOutput {
        ScannedTemperature(f64, std::time::Instant),
        SampledPedestrian(f64, std::time::Instant),
    }
    use RuntimeOutput as O;
    #[derive(Debug)]
    pub struct RuntimeInit {
        pub temperature: f64,
    }
    #[derive(PartialEq)]
    pub enum RuntimeTimer {
        PeriodSampledPedestrian,
        PeriodScannedTemperature,
        DelayScanSample,
        TimeoutScanSample,
    }
    use RuntimeTimer as T;
    impl timer_stream::Timing for RuntimeTimer {
        fn get_duration(&self) -> std::time::Duration {
            match self {
                T::PeriodSampledPedestrian => std::time::Duration::from_millis(250u64),
                T::PeriodScannedTemperature => std::time::Duration::from_millis(100u64),
                T::DelayScanSample => std::time::Duration::from_millis(10u64),
                T::TimeoutScanSample => std::time::Duration::from_millis(3000u64),
            }
        }
        fn do_reset(&self) -> bool {
            match self {
                T::PeriodSampledPedestrian => false,
                T::PeriodScannedTemperature => false,
                T::DelayScanSample => true,
                T::TimeoutScanSample => true,
            }
        }
    }
    pub struct Runtime {
        scan_sample: scan_sample_service::ScanSampleService,
        output: futures::channel::mpsc::Sender<O>,
        timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
    }
    impl Runtime {
        pub fn new(
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        ) -> Runtime {
            let scan_sample =
                scan_sample_service::ScanSampleService::init(output.clone(), timer.clone());
            Runtime {
                scan_sample,
                output,
                timer,
            }
        }
        #[inline]
        pub async fn send_timer(
            &mut self,
            timer: T,
            instant: std::time::Instant,
        ) -> Result<(), futures::channel::mpsc::SendError> {
            self.timer.send((timer, instant)).await?;
            Ok(())
        }
        pub async fn run_loop(
            self,
            _grust_reserved_init_instant: std::time::Instant,
            input: impl futures::Stream<Item = I>,
            init_vals: RuntimeInit,
        ) -> Result<(), futures::channel::mpsc::SendError> {
            futures::pin_mut!(input);
            let mut runtime = self;
            let RuntimeInit { temperature } = init_vals;
            runtime
                .scan_sample
                .handle_init(_grust_reserved_init_instant, temperature)
                .await?;
            while let Some(input) = input.next().await {
                match input {
                    I::Temperature(temperature, _grust_reserved_instant) => {
                        runtime
                            .scan_sample
                            .handle_temperature(_grust_reserved_instant, temperature)
                            .await?;
                    }
                    I::Timer(T::DelayScanSample, _grust_reserved_instant) => {
                        runtime
                            .scan_sample
                            .handle_delay_scan_sample(_grust_reserved_instant)
                            .await?;
                    }
                    I::Pedestrian(pedestrian, _grust_reserved_instant) => {
                        runtime
                            .scan_sample
                            .handle_pedestrian(_grust_reserved_instant, pedestrian)
                            .await?;
                    }
                    I::Timer(T::TimeoutScanSample, _grust_reserved_instant) => {
                        runtime
                            .scan_sample
                            .handle_timeout_scan_sample(_grust_reserved_instant)
                            .await?;
                    }
                    I::Timer(T::PeriodScannedTemperature, _grust_reserved_instant) => {
                        runtime
                            .scan_sample
                            .handle_period_scanned_temperature(_grust_reserved_instant)
                            .await?;
                    }
                    I::Timer(T::PeriodSampledPedestrian, _grust_reserved_instant) => {
                        runtime
                            .scan_sample
                            .handle_period_sampled_pedestrian(_grust_reserved_instant)
                            .await?;
                    }
                }
            }
            Ok(())
        }
    }
    pub mod scan_sample_service {
        use super::*;
        use futures::{sink::SinkExt, stream::StreamExt};
        mod ctx_ty {
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct ScannedTemperature(f64, bool);
            impl ScannedTemperature {
                pub fn set(&mut self, scanned_temperature: f64) {
                    self.1 = self.0 != scanned_temperature;
                    self.0 = scanned_temperature;
                }
                pub fn get(&self) -> f64 {
                    self.0
                }
                pub fn take(&mut self) -> f64 {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct Temperature(f64, bool);
            impl Temperature {
                pub fn set(&mut self, temperature: f64) {
                    self.1 = self.0 != temperature;
                    self.0 = temperature;
                }
                pub fn get(&self) -> f64 {
                    self.0
                }
                pub fn take(&mut self) -> f64 {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct Pedestrian(Option<f64>, bool);
            impl Pedestrian {
                pub fn set(&mut self, pedestrian: Option<f64>) {
                    self.1 = self.0 != pedestrian;
                    self.0 = pedestrian;
                }
                pub fn get(&self) -> Option<f64> {
                    self.0
                }
                pub fn take(&mut self) -> Option<f64> {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct SampledPedestrian(Option<f64>, bool);
            impl SampledPedestrian {
                pub fn set(&mut self, sampled_pedestrian: Option<f64>) {
                    self.1 = self.0 != sampled_pedestrian;
                    self.0 = sampled_pedestrian;
                }
                pub fn get(&self) -> Option<f64> {
                    self.0
                }
                pub fn take(&mut self) -> Option<f64> {
                    std::mem::take(&mut self.0)
                }
                pub fn is_new(&self) -> bool {
                    self.1
                }
                pub fn reset(&mut self) {
                    self.1 = false;
                }
            }
        }
        #[derive(Clone, Copy, PartialEq, Default, Debug)]
        pub struct Context {
            pub scanned_temperature: ctx_ty::ScannedTemperature,
            pub temperature: ctx_ty::Temperature,
            pub pedestrian: ctx_ty::Pedestrian,
            pub sampled_pedestrian: ctx_ty::SampledPedestrian,
        }
        impl Context {
            fn init() -> Context {
                Default::default()
            }
            fn reset(&mut self) {
                self.scanned_temperature.reset();
                self.temperature.reset();
                self.pedestrian.reset();
                self.sampled_pedestrian.reset();
            }
        }
        #[derive(Default)]
        pub struct ScanSampleServiceStore {
            period_sampled_pedestrian: Option<((), std::time::Instant)>,
            period_scanned_temperature: Option<((), std::time::Instant)>,
            temperature: Option<(f64, std::time::Instant)>,
            pedestrian: Option<(f64, std::time::Instant)>,
        }
        impl ScanSampleServiceStore {
            pub fn not_empty(&self) -> bool {
                self.period_sampled_pedestrian.is_some()
                    || self.period_scanned_temperature.is_some()
                    || self.temperature.is_some()
                    || self.pedestrian.is_some()
            }
        }
        pub struct ScanSampleService {
            begin: std::time::Instant,
            context: Context,
            delayed: bool,
            input_store: ScanSampleServiceStore,
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        }
        impl ScanSampleService {
            pub fn init(
                output: futures::channel::mpsc::Sender<O>,
                timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
            ) -> ScanSampleService {
                let context = Context::init();
                let delayed = true;
                let input_store = Default::default();
                ScanSampleService {
                    begin: std::time::Instant::now(),
                    context,
                    delayed,
                    input_store,
                    output,
                    timer,
                }
            }
            pub async fn handle_init(
                &mut self,
                _grust_reserved_instant: std::time::Instant,
                temperature: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_service_timeout(_grust_reserved_instant).await?;
                let sampled_pedestrian_ref = &mut None;
                self.context.temperature.set(temperature);
                self.send_timer(T::PeriodScannedTemperature, _grust_reserved_instant)
                    .await?;
                self.context.scanned_temperature.set(temperature);
                self.send_timer(T::PeriodSampledPedestrian, _grust_reserved_instant)
                    .await?;
                *sampled_pedestrian_ref = self.context.pedestrian.take();
                self.send_output(
                    O::ScannedTemperature(
                        self.context.scanned_temperature.get(),
                        _grust_reserved_instant,
                    ),
                    _grust_reserved_instant,
                )
                .await?;
                if let Some(sampled_pedestrian) = *sampled_pedestrian_ref {
                    self.send_output(
                        O::SampledPedestrian(sampled_pedestrian, _grust_reserved_instant),
                        _grust_reserved_instant,
                    )
                    .await?;
                }
                Ok(())
            }
            pub async fn handle_timeout_scan_sample(
                &mut self,
                _timeout_scan_sample_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_time_constraints(_timeout_scan_sample_instant)
                    .await?;
                self.context.reset();
                self.send_output(
                    O::ScannedTemperature(
                        self.context.scanned_temperature.get(),
                        _timeout_scan_sample_instant,
                    ),
                    _timeout_scan_sample_instant,
                )
                .await?;
                Ok(())
            }
            #[inline]
            pub async fn reset_service_timeout(
                &mut self,
                _timeout_scan_sample_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer
                    .send((T::TimeoutScanSample, _timeout_scan_sample_instant))
                    .await?;
                Ok(())
            }
            pub async fn handle_period_sampled_pedestrian(
                &mut self,
                _period_sampled_pedestrian_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_period_sampled_pedestrian_instant)
                        .await?;
                    self.context.reset();
                    let sampled_pedestrian_ref = &mut None;
                    self.send_timer(
                        T::PeriodSampledPedestrian,
                        _period_sampled_pedestrian_instant,
                    )
                    .await?;
                    *sampled_pedestrian_ref = self.context.pedestrian.take();
                    if let Some(sampled_pedestrian) = *sampled_pedestrian_ref {
                        self.send_output(
                            O::SampledPedestrian(
                                sampled_pedestrian,
                                _period_sampled_pedestrian_instant,
                            ),
                            _period_sampled_pedestrian_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .period_sampled_pedestrian
                        .replace(((), _period_sampled_pedestrian_instant));
                    assert!
                    (unique.is_none(),
                    "flow `period_sampled_pedestrian` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            pub async fn handle_period_scanned_temperature(
                &mut self,
                _period_scanned_temperature_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_period_scanned_temperature_instant)
                        .await?;
                    self.context.reset();
                    self.send_timer(
                        T::PeriodScannedTemperature,
                        _period_scanned_temperature_instant,
                    )
                    .await?;
                    self.context
                        .scanned_temperature
                        .set(self.context.temperature.get());
                    if self.context.scanned_temperature.is_new() {
                        self.send_output(
                            O::ScannedTemperature(
                                self.context.scanned_temperature.get(),
                                _period_scanned_temperature_instant,
                            ),
                            _period_scanned_temperature_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .period_scanned_temperature
                        .replace(((), _period_scanned_temperature_instant));
                    assert!
                    (unique.is_none(),
                    "flow `period_scanned_temperature` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            pub async fn handle_temperature(
                &mut self,
                _temperature_instant: std::time::Instant,
                temperature: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_temperature_instant).await?;
                    self.context.reset();
                    self.context.temperature.set(temperature);
                    if self.context.scanned_temperature.is_new() {
                        self.send_output(
                            O::ScannedTemperature(
                                self.context.scanned_temperature.get(),
                                _temperature_instant,
                            ),
                            _temperature_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .temperature
                        .replace((temperature, _temperature_instant));
                    assert!
                    (unique.is_none(),
                    "flow `temperature` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            pub async fn handle_delay_scan_sample(
                &mut self,
                _grust_reserved_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.context.reset();
                if self.input_store.not_empty() {
                    self.reset_time_constraints(_grust_reserved_instant).await?;
                    match (
                        self.input_store.period_sampled_pedestrian.take(),
                        self.input_store.period_scanned_temperature.take(),
                        self.input_store.temperature.take(),
                        self.input_store.pedestrian.take(),
                    ) {
                        (None, None, None, None) => {}
                        (Some(((), _period_sampled_pedestrian_instant)), None, None, None) => {
                            let sampled_pedestrian_ref = &mut None;
                            self.send_timer(
                                T::PeriodSampledPedestrian,
                                _period_sampled_pedestrian_instant,
                            )
                            .await?;
                            *sampled_pedestrian_ref = self.context.pedestrian.take();
                            if let Some(sampled_pedestrian) = *sampled_pedestrian_ref {
                                self.send_output(
                                    O::SampledPedestrian(
                                        sampled_pedestrian,
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (None, Some(((), _period_scanned_temperature_instant)), None, None) => {
                            self.send_timer(
                                T::PeriodScannedTemperature,
                                _period_scanned_temperature_instant,
                            )
                            .await?;
                            self.context
                                .scanned_temperature
                                .set(self.context.temperature.get());
                            if self.context.scanned_temperature.is_new() {
                                self.send_output(
                                    O::ScannedTemperature(
                                        self.context.scanned_temperature.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some(((), _period_sampled_pedestrian_instant)),
                            Some(((), _period_scanned_temperature_instant)),
                            None,
                            None,
                        ) => {
                            let sampled_pedestrian_ref = &mut None;
                            self.send_timer(
                                T::PeriodScannedTemperature,
                                _period_scanned_temperature_instant,
                            )
                            .await?;
                            self.context
                                .scanned_temperature
                                .set(self.context.temperature.get());
                            if self.context.scanned_temperature.is_new() {
                                self.send_output(
                                    O::ScannedTemperature(
                                        self.context.scanned_temperature.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.send_timer(
                                T::PeriodSampledPedestrian,
                                _period_sampled_pedestrian_instant,
                            )
                            .await?;
                            *sampled_pedestrian_ref = self.context.pedestrian.take();
                            if let Some(sampled_pedestrian) = *sampled_pedestrian_ref {
                                self.send_output(
                                    O::SampledPedestrian(
                                        sampled_pedestrian,
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (None, None, Some((temperature, _temperature_instant)), None) => {
                            self.context.temperature.set(temperature);
                            if self.context.scanned_temperature.is_new() {
                                self.send_output(
                                    O::ScannedTemperature(
                                        self.context.scanned_temperature.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some(((), _period_sampled_pedestrian_instant)),
                            None,
                            Some((temperature, _temperature_instant)),
                            None,
                        ) => {
                            let sampled_pedestrian_ref = &mut None;
                            self.context.temperature.set(temperature);
                            if self.context.scanned_temperature.is_new() {
                                self.send_output(
                                    O::ScannedTemperature(
                                        self.context.scanned_temperature.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.send_timer(
                                T::PeriodSampledPedestrian,
                                _period_sampled_pedestrian_instant,
                            )
                            .await?;
                            *sampled_pedestrian_ref = self.context.pedestrian.take();
                            if let Some(sampled_pedestrian) = *sampled_pedestrian_ref {
                                self.send_output(
                                    O::SampledPedestrian(
                                        sampled_pedestrian,
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some(((), _period_scanned_temperature_instant)),
                            Some((temperature, _temperature_instant)),
                            None,
                        ) => {
                            self.context.temperature.set(temperature);
                            self.send_timer(
                                T::PeriodScannedTemperature,
                                _period_scanned_temperature_instant,
                            )
                            .await?;
                            self.context.scanned_temperature.set(temperature);
                            if self.context.scanned_temperature.is_new() {
                                self.send_output(
                                    O::ScannedTemperature(
                                        self.context.scanned_temperature.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some(((), _period_sampled_pedestrian_instant)),
                            Some(((), _period_scanned_temperature_instant)),
                            Some((temperature, _temperature_instant)),
                            None,
                        ) => {
                            let sampled_pedestrian_ref = &mut None;
                            self.context.temperature.set(temperature);
                            self.send_timer(
                                T::PeriodScannedTemperature,
                                _period_scanned_temperature_instant,
                            )
                            .await?;
                            self.context.scanned_temperature.set(temperature);
                            if self.context.scanned_temperature.is_new() {
                                self.send_output(
                                    O::ScannedTemperature(
                                        self.context.scanned_temperature.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.send_timer(
                                T::PeriodSampledPedestrian,
                                _period_sampled_pedestrian_instant,
                            )
                            .await?;
                            *sampled_pedestrian_ref = self.context.pedestrian.take();
                            if let Some(sampled_pedestrian) = *sampled_pedestrian_ref {
                                self.send_output(
                                    O::SampledPedestrian(
                                        sampled_pedestrian,
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (None, None, None, Some((pedestrian, _pedestrian_instant))) => {
                            let pedestrian_ref = &mut None;
                            *pedestrian_ref = Some(pedestrian);
                            if pedestrian_ref.is_some() {
                                self.context.pedestrian.set(*pedestrian_ref);
                            }
                        }
                        (
                            Some(((), _period_sampled_pedestrian_instant)),
                            None,
                            None,
                            Some((pedestrian, _pedestrian_instant)),
                        ) => {
                            let pedestrian_ref = &mut None;
                            let sampled_pedestrian_ref = &mut None;
                            *pedestrian_ref = Some(pedestrian);
                            self.send_timer(
                                T::PeriodSampledPedestrian,
                                _period_sampled_pedestrian_instant,
                            )
                            .await?;
                            if pedestrian_ref.is_some() {
                                self.context.pedestrian.set(*pedestrian_ref);
                            }
                            *sampled_pedestrian_ref = self.context.pedestrian.take();
                            if let Some(sampled_pedestrian) = *sampled_pedestrian_ref {
                                self.send_output(
                                    O::SampledPedestrian(
                                        sampled_pedestrian,
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some(((), _period_scanned_temperature_instant)),
                            None,
                            Some((pedestrian, _pedestrian_instant)),
                        ) => {
                            let pedestrian_ref = &mut None;
                            *pedestrian_ref = Some(pedestrian);
                            if pedestrian_ref.is_some() {
                                self.context.pedestrian.set(*pedestrian_ref);
                            }
                            self.send_timer(
                                T::PeriodScannedTemperature,
                                _period_scanned_temperature_instant,
                            )
                            .await?;
                            self.context
                                .scanned_temperature
                                .set(self.context.temperature.get());
                            if self.context.scanned_temperature.is_new() {
                                self.send_output(
                                    O::ScannedTemperature(
                                        self.context.scanned_temperature.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some(((), _period_sampled_pedestrian_instant)),
                            Some(((), _period_scanned_temperature_instant)),
                            None,
                            Some((pedestrian, _pedestrian_instant)),
                        ) => {
                            let pedestrian_ref = &mut None;
                            let sampled_pedestrian_ref = &mut None;
                            *pedestrian_ref = Some(pedestrian);
                            self.send_timer(
                                T::PeriodScannedTemperature,
                                _period_scanned_temperature_instant,
                            )
                            .await?;
                            self.context
                                .scanned_temperature
                                .set(self.context.temperature.get());
                            if self.context.scanned_temperature.is_new() {
                                self.send_output(
                                    O::ScannedTemperature(
                                        self.context.scanned_temperature.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.send_timer(
                                T::PeriodSampledPedestrian,
                                _period_sampled_pedestrian_instant,
                            )
                            .await?;
                            if pedestrian_ref.is_some() {
                                self.context.pedestrian.set(*pedestrian_ref);
                            }
                            *sampled_pedestrian_ref = self.context.pedestrian.take();
                            if let Some(sampled_pedestrian) = *sampled_pedestrian_ref {
                                self.send_output(
                                    O::SampledPedestrian(
                                        sampled_pedestrian,
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            None,
                            Some((temperature, _temperature_instant)),
                            Some((pedestrian, _pedestrian_instant)),
                        ) => {
                            let pedestrian_ref = &mut None;
                            *pedestrian_ref = Some(pedestrian);
                            if pedestrian_ref.is_some() {
                                self.context.pedestrian.set(*pedestrian_ref);
                            }
                            self.context.temperature.set(temperature);
                            if self.context.scanned_temperature.is_new() {
                                self.send_output(
                                    O::ScannedTemperature(
                                        self.context.scanned_temperature.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some(((), _period_sampled_pedestrian_instant)),
                            None,
                            Some((temperature, _temperature_instant)),
                            Some((pedestrian, _pedestrian_instant)),
                        ) => {
                            let pedestrian_ref = &mut None;
                            let sampled_pedestrian_ref = &mut None;
                            *pedestrian_ref = Some(pedestrian);
                            self.context.temperature.set(temperature);
                            if self.context.scanned_temperature.is_new() {
                                self.send_output(
                                    O::ScannedTemperature(
                                        self.context.scanned_temperature.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.send_timer(
                                T::PeriodSampledPedestrian,
                                _period_sampled_pedestrian_instant,
                            )
                            .await?;
                            if pedestrian_ref.is_some() {
                                self.context.pedestrian.set(*pedestrian_ref);
                            }
                            *sampled_pedestrian_ref = self.context.pedestrian.take();
                            if let Some(sampled_pedestrian) = *sampled_pedestrian_ref {
                                self.send_output(
                                    O::SampledPedestrian(
                                        sampled_pedestrian,
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some(((), _period_scanned_temperature_instant)),
                            Some((temperature, _temperature_instant)),
                            Some((pedestrian, _pedestrian_instant)),
                        ) => {
                            let pedestrian_ref = &mut None;
                            *pedestrian_ref = Some(pedestrian);
                            if pedestrian_ref.is_some() {
                                self.context.pedestrian.set(*pedestrian_ref);
                            }
                            self.context.temperature.set(temperature);
                            self.send_timer(
                                T::PeriodScannedTemperature,
                                _period_scanned_temperature_instant,
                            )
                            .await?;
                            self.context.scanned_temperature.set(temperature);
                            if self.context.scanned_temperature.is_new() {
                                self.send_output(
                                    O::ScannedTemperature(
                                        self.context.scanned_temperature.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some(((), _period_sampled_pedestrian_instant)),
                            Some(((), _period_scanned_temperature_instant)),
                            Some((temperature, _temperature_instant)),
                            Some((pedestrian, _pedestrian_instant)),
                        ) => {
                            let pedestrian_ref = &mut None;
                            let sampled_pedestrian_ref = &mut None;
                            *pedestrian_ref = Some(pedestrian);
                            self.context.temperature.set(temperature);
                            self.send_timer(
                                T::PeriodScannedTemperature,
                                _period_scanned_temperature_instant,
                            )
                            .await?;
                            self.context.scanned_temperature.set(temperature);
                            if self.context.scanned_temperature.is_new() {
                                self.send_output(
                                    O::ScannedTemperature(
                                        self.context.scanned_temperature.get(),
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            self.send_timer(
                                T::PeriodSampledPedestrian,
                                _period_sampled_pedestrian_instant,
                            )
                            .await?;
                            if pedestrian_ref.is_some() {
                                self.context.pedestrian.set(*pedestrian_ref);
                            }
                            *sampled_pedestrian_ref = self.context.pedestrian.take();
                            if let Some(sampled_pedestrian) = *sampled_pedestrian_ref {
                                self.send_output(
                                    O::SampledPedestrian(
                                        sampled_pedestrian,
                                        _grust_reserved_instant,
                                    ),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                    }
                } else {
                    self.delayed = true;
                }
                Ok(())
            }
            #[inline]
            pub async fn reset_service_delay(
                &mut self,
                _grust_reserved_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer
                    .send((T::DelayScanSample, _grust_reserved_instant))
                    .await?;
                Ok(())
            }
            pub async fn handle_pedestrian(
                &mut self,
                _pedestrian_instant: std::time::Instant,
                pedestrian: f64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_pedestrian_instant).await?;
                    self.context.reset();
                    let pedestrian_ref = &mut None;
                    *pedestrian_ref = Some(pedestrian);
                    if pedestrian_ref.is_some() {
                        self.context.pedestrian.set(*pedestrian_ref);
                    }
                } else {
                    let unique = self
                        .input_store
                        .pedestrian
                        .replace((pedestrian, _pedestrian_instant));
                    assert!
                    (unique.is_none(),
                    "flow `pedestrian` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            #[inline]
            pub async fn reset_time_constraints(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_service_delay(instant).await?;
                self.delayed = false;
                Ok(())
            }
            #[inline]
            pub async fn send_output(
                &mut self,
                output: O,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_service_timeout(instant).await?;
                self.output.feed(output).await?;
                Ok(())
            }
            #[inline]
            pub async fn send_timer(
                &mut self,
                timer: T,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer.feed((timer, instant)).await?;
                Ok(())
            }
        }
    }
}
