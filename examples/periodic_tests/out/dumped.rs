pub mod runtime {
    use super::*;
    use futures::{sink::SinkExt, stream::StreamExt};
    pub enum RuntimeInput {
        InputS(i64, std::time::Instant),
        InputE(i64, std::time::Instant),
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
                (I::InputS(this, _), I::InputS(other, _)) => this.eq(other),
                (I::InputE(this, _), I::InputE(other, _)) => this.eq(other),
                (I::Timer(this, _), I::Timer(other, _)) => this.eq(other),
                _ => false,
            }
        }
    }
    impl RuntimeInput {
        pub fn get_instant(&self) -> std::time::Instant {
            match self {
                I::InputS(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::InputE(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::Timer(_, _grust_reserved_instant) => *_grust_reserved_instant,
            }
        }
        pub fn order(v1: &Self, v2: &Self) -> std::cmp::Ordering {
            v1.get_instant().cmp(&v2.get_instant())
        }
    }
    #[derive(Debug, PartialEq)]
    pub enum RuntimeOutput {
        Scanned(i64, std::time::Instant),
        Sampled(i64, std::time::Instant),
    }
    use RuntimeOutput as O;
    #[derive(Debug, Default)]
    pub struct RuntimeInit {
        pub input_s: i64,
    }
    #[derive(PartialEq)]
    pub enum RuntimeTimer {
        PeriodClock,
        DelayTest,
        TimeoutTest,
    }
    use RuntimeTimer as T;
    impl timer_stream::Timing for RuntimeTimer {
        fn get_duration(&self) -> std::time::Duration {
            match self {
                T::PeriodClock => std::time::Duration::from_millis(100u64),
                T::DelayTest => std::time::Duration::from_millis(10u64),
                T::TimeoutTest => std::time::Duration::from_millis(2000u64),
            }
        }
        fn do_reset(&self) -> bool {
            match self {
                T::PeriodClock => false,
                T::DelayTest => true,
                T::TimeoutTest => true,
            }
        }
    }
    pub struct Runtime {
        test: test_service::TestService,
        output: futures::channel::mpsc::Sender<O>,
        timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
    }
    impl Runtime {
        pub fn new(
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        ) -> Runtime {
            let test = test_service::TestService::init(output.clone(), timer.clone());
            Runtime {
                test,
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
            let RuntimeInit { input_s } = init_vals;
            runtime
                .test
                .handle_init(_grust_reserved_init_instant, input_s)
                .await?;
            while let Some(input) = input.next().await {
                match input {
                    I::InputE(input_e, _grust_reserved_instant) => {
                        runtime
                            .test
                            .handle_input_e(_grust_reserved_instant, input_e)
                            .await?;
                    }
                    I::Timer(T::PeriodClock, _grust_reserved_instant) => {
                        runtime
                            .test
                            .handle_period_clock(_grust_reserved_instant)
                            .await?;
                    }
                    I::Timer(T::DelayTest, _grust_reserved_instant) => {
                        runtime
                            .test
                            .handle_delay_test(_grust_reserved_instant)
                            .await?;
                    }
                    I::InputS(input_s, _grust_reserved_instant) => {
                        runtime
                            .test
                            .handle_input_s(_grust_reserved_instant, input_s)
                            .await?;
                    }
                    I::Timer(T::TimeoutTest, _grust_reserved_instant) => {
                        runtime
                            .test
                            .handle_timeout_test(_grust_reserved_instant)
                            .await?;
                    }
                }
            }
            Ok(())
        }
    }
    pub mod test_service {
        use super::*;
        use futures::{sink::SinkExt, stream::StreamExt};
        mod ctx_ty {
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct Scanned(i64, bool);
            impl Scanned {
                pub fn set(&mut self, scanned: i64) {
                    self.1 = self.0 != scanned;
                    self.0 = scanned;
                }
                pub fn get(&self) -> i64 {
                    self.0
                }
                pub fn take(&mut self) -> i64 {
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
            pub struct Sampled(Option<i64>, bool);
            impl Sampled {
                pub fn set(&mut self, sampled: Option<i64>) {
                    self.1 = self.0 != sampled;
                    self.0 = sampled;
                }
                pub fn get(&self) -> Option<i64> {
                    self.0
                }
                pub fn take(&mut self) -> Option<i64> {
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
            pub struct InputE(Option<i64>, bool);
            impl InputE {
                pub fn set(&mut self, input_e: Option<i64>) {
                    self.1 = self.0 != input_e;
                    self.0 = input_e;
                }
                pub fn get(&self) -> Option<i64> {
                    self.0
                }
                pub fn take(&mut self) -> Option<i64> {
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
            pub struct InputS(i64, bool);
            impl InputS {
                pub fn set(&mut self, input_s: i64) {
                    self.1 = self.0 != input_s;
                    self.0 = input_s;
                }
                pub fn get(&self) -> i64 {
                    self.0
                }
                pub fn take(&mut self) -> i64 {
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
            pub scanned: ctx_ty::Scanned,
            pub sampled: ctx_ty::Sampled,
            pub input_e: ctx_ty::InputE,
            pub input_s: ctx_ty::InputS,
        }
        impl Context {
            fn init() -> Context {
                Default::default()
            }
            fn reset(&mut self) {
                self.scanned.reset();
                self.sampled.reset();
                self.input_e.reset();
                self.input_s.reset();
            }
        }
        #[derive(Default)]
        pub struct TestServiceStore {
            period_clock: Option<((), std::time::Instant)>,
            input_s: Option<(i64, std::time::Instant)>,
            input_e: Option<(i64, std::time::Instant)>,
        }
        impl TestServiceStore {
            pub fn not_empty(&self) -> bool {
                self.period_clock.is_some() || self.input_s.is_some() || self.input_e.is_some()
            }
        }
        pub struct TestService {
            begin: std::time::Instant,
            context: Context,
            delayed: bool,
            input_store: TestServiceStore,
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        }
        impl TestService {
            pub fn init(
                output: futures::channel::mpsc::Sender<O>,
                timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
            ) -> TestService {
                let context = Context::init();
                let delayed = true;
                let input_store = Default::default();
                TestService {
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
                input_s: i64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_service_timeout(_grust_reserved_instant).await?;
                let clock_ref = &mut None;
                let sampled_ref = &mut None;
                self.context.input_s.set(input_s);
                self.send_timer(T::PeriodClock, _grust_reserved_instant)
                    .await?;
                *clock_ref = Some(
                    (_grust_reserved_instant
                        .duration_since(self.begin)
                        .as_millis()) as f64,
                );
                if clock_ref.is_some() {
                    *sampled_ref = self.context.input_e.take();
                }
                if let Some(sampled) = *sampled_ref {
                    self.send_output(
                        O::Sampled(sampled, _grust_reserved_instant),
                        _grust_reserved_instant,
                    )
                    .await?;
                }
                if clock_ref.is_some() {
                    self.context.scanned.set(input_s);
                }
                self.send_output(
                    O::Scanned(self.context.scanned.get(), _grust_reserved_instant),
                    _grust_reserved_instant,
                )
                .await?;
                Ok(())
            }
            pub async fn handle_period_clock(
                &mut self,
                _period_clock_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_period_clock_instant).await?;
                    self.context.reset();
                    let clock_ref = &mut None;
                    let sampled_ref = &mut None;
                    self.send_timer(T::PeriodClock, _period_clock_instant)
                        .await?;
                    *clock_ref =
                        Some((_period_clock_instant.duration_since(self.begin).as_millis()) as f64);
                    if clock_ref.is_some() {
                        *sampled_ref = self.context.input_e.take();
                    }
                    if let Some(sampled) = *sampled_ref {
                        self.send_output(
                            O::Sampled(sampled, _period_clock_instant),
                            _period_clock_instant,
                        )
                        .await?;
                    }
                    if clock_ref.is_some() {
                        self.context.scanned.set(self.context.input_s.get());
                    }
                    if self.context.scanned.is_new() {
                        self.send_output(
                            O::Scanned(self.context.scanned.get(), _period_clock_instant),
                            _period_clock_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .period_clock
                        .replace(((), _period_clock_instant));
                    assert!
                    (unique.is_none(),
                    "flow `period_clock` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            pub async fn handle_delay_test(
                &mut self,
                _grust_reserved_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.context.reset();
                if self.input_store.not_empty() {
                    self.reset_time_constraints(_grust_reserved_instant).await?;
                    match (
                        self.input_store.period_clock.take(),
                        self.input_store.input_s.take(),
                        self.input_store.input_e.take(),
                    ) {
                        (None, None, None) => {}
                        (Some(((), _period_clock_instant)), None, None) => {
                            let clock_ref = &mut None;
                            let sampled_ref = &mut None;
                            self.send_timer(T::PeriodClock, _period_clock_instant)
                                .await?;
                            *clock_ref = Some(
                                (_grust_reserved_instant
                                    .duration_since(self.begin)
                                    .as_millis()) as f64,
                            );
                            if clock_ref.is_some() {
                                *sampled_ref = self.context.input_e.take();
                            }
                            if let Some(sampled) = *sampled_ref {
                                self.send_output(
                                    O::Sampled(sampled, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            if clock_ref.is_some() {
                                self.context.scanned.set(self.context.input_s.get());
                            }
                            if self.context.scanned.is_new() {
                                self.send_output(
                                    O::Scanned(self.context.scanned.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (None, Some((input_s, _input_s_instant)), None) => {
                            self.context.input_s.set(input_s);
                            if self.context.scanned.is_new() {
                                self.send_output(
                                    O::Scanned(self.context.scanned.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some(((), _period_clock_instant)),
                            Some((input_s, _input_s_instant)),
                            None,
                        ) => {
                            let clock_ref = &mut None;
                            let sampled_ref = &mut None;
                            self.context.input_s.set(input_s);
                            self.send_timer(T::PeriodClock, _period_clock_instant)
                                .await?;
                            *clock_ref = Some(
                                (_grust_reserved_instant
                                    .duration_since(self.begin)
                                    .as_millis()) as f64,
                            );
                            if clock_ref.is_some() {
                                *sampled_ref = self.context.input_e.take();
                            }
                            if let Some(sampled) = *sampled_ref {
                                self.send_output(
                                    O::Sampled(sampled, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            if clock_ref.is_some() {
                                self.context.scanned.set(input_s);
                            }
                            if self.context.scanned.is_new() {
                                self.send_output(
                                    O::Scanned(self.context.scanned.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (None, None, Some((input_e, _input_e_instant))) => {
                            let input_e_ref = &mut None;
                            *input_e_ref = Some(input_e);
                            if input_e_ref.is_some() {
                                self.context.input_e.set(*input_e_ref);
                            }
                        }
                        (
                            Some(((), _period_clock_instant)),
                            None,
                            Some((input_e, _input_e_instant)),
                        ) => {
                            let input_e_ref = &mut None;
                            let clock_ref = &mut None;
                            let sampled_ref = &mut None;
                            *input_e_ref = Some(input_e);
                            self.send_timer(T::PeriodClock, _period_clock_instant)
                                .await?;
                            *clock_ref = Some(
                                (_grust_reserved_instant
                                    .duration_since(self.begin)
                                    .as_millis()) as f64,
                            );
                            if input_e_ref.is_some() {
                                self.context.input_e.set(*input_e_ref);
                            }
                            if clock_ref.is_some() {
                                *sampled_ref = self.context.input_e.take();
                            }
                            if let Some(sampled) = *sampled_ref {
                                self.send_output(
                                    O::Sampled(sampled, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            if clock_ref.is_some() {
                                self.context.scanned.set(self.context.input_s.get());
                            }
                            if self.context.scanned.is_new() {
                                self.send_output(
                                    O::Scanned(self.context.scanned.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            None,
                            Some((input_s, _input_s_instant)),
                            Some((input_e, _input_e_instant)),
                        ) => {
                            let input_e_ref = &mut None;
                            *input_e_ref = Some(input_e);
                            if input_e_ref.is_some() {
                                self.context.input_e.set(*input_e_ref);
                            }
                            self.context.input_s.set(input_s);
                            if self.context.scanned.is_new() {
                                self.send_output(
                                    O::Scanned(self.context.scanned.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some(((), _period_clock_instant)),
                            Some((input_s, _input_s_instant)),
                            Some((input_e, _input_e_instant)),
                        ) => {
                            let input_e_ref = &mut None;
                            let clock_ref = &mut None;
                            let sampled_ref = &mut None;
                            *input_e_ref = Some(input_e);
                            self.context.input_s.set(input_s);
                            self.send_timer(T::PeriodClock, _period_clock_instant)
                                .await?;
                            *clock_ref = Some(
                                (_grust_reserved_instant
                                    .duration_since(self.begin)
                                    .as_millis()) as f64,
                            );
                            if input_e_ref.is_some() {
                                self.context.input_e.set(*input_e_ref);
                            }
                            if clock_ref.is_some() {
                                *sampled_ref = self.context.input_e.take();
                            }
                            if let Some(sampled) = *sampled_ref {
                                self.send_output(
                                    O::Sampled(sampled, _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            if clock_ref.is_some() {
                                self.context.scanned.set(input_s);
                            }
                            if self.context.scanned.is_new() {
                                self.send_output(
                                    O::Scanned(self.context.scanned.get(), _grust_reserved_instant),
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
                    .send((T::DelayTest, _grust_reserved_instant))
                    .await?;
                Ok(())
            }
            pub async fn handle_input_s(
                &mut self,
                _input_s_instant: std::time::Instant,
                input_s: i64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_input_s_instant).await?;
                    self.context.reset();
                    self.context.input_s.set(input_s);
                    if self.context.scanned.is_new() {
                        self.send_output(
                            O::Scanned(self.context.scanned.get(), _input_s_instant),
                            _input_s_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self
                        .input_store
                        .input_s
                        .replace((input_s, _input_s_instant));
                    assert!
                    (unique.is_none(),
                    "flow `input_s` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            pub async fn handle_timeout_test(
                &mut self,
                _timeout_test_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_time_constraints(_timeout_test_instant).await?;
                self.context.reset();
                self.send_output(
                    O::Scanned(self.context.scanned.get(), _timeout_test_instant),
                    _timeout_test_instant,
                )
                .await?;
                Ok(())
            }
            #[inline]
            pub async fn reset_service_timeout(
                &mut self,
                _timeout_test_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer
                    .send((T::TimeoutTest, _timeout_test_instant))
                    .await?;
                Ok(())
            }
            pub async fn handle_input_e(
                &mut self,
                _input_e_instant: std::time::Instant,
                input_e: i64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_input_e_instant).await?;
                    self.context.reset();
                    let input_e_ref = &mut None;
                    *input_e_ref = Some(input_e);
                    if input_e_ref.is_some() {
                        self.context.input_e.set(*input_e_ref);
                    }
                } else {
                    let unique = self
                        .input_store
                        .input_e
                        .replace((input_e, _input_e_instant));
                    assert!
                    (unique.is_none(),
                    "flow `input_e` changes twice within one minimal delay of the service, consider reducing this delay");
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
