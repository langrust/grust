pub mod runtime {
    use super::*;
    use futures::{sink::SinkExt, stream::StreamExt};
    pub enum RuntimeInput {
        Reset(bool, std::time::Instant),
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
                (I::Reset(this, _), I::Reset(other, _)) => this.eq(other),
                (I::Timer(this, _), I::Timer(other, _)) => this.eq(other),
                _ => false,
            }
        }
    }
    impl RuntimeInput {
        pub fn get_instant(&self) -> std::time::Instant {
            match self {
                I::Reset(_, _grust_reserved_instant) => *_grust_reserved_instant,
                I::Timer(_, _grust_reserved_instant) => *_grust_reserved_instant,
            }
        }
        pub fn order(v1: &Self, v2: &Self) -> std::cmp::Ordering {
            v1.get_instant().cmp(&v2.get_instant())
        }
    }
    #[derive(Debug, PartialEq)]
    pub enum RuntimeOutput {
        O2(i64, std::time::Instant),
        O1(i64, std::time::Instant),
    }
    use RuntimeOutput as O;
    #[derive(Debug, Default)]
    pub struct RuntimeInit {
        pub reset: bool,
    }
    #[derive(PartialEq)]
    pub enum RuntimeTimer {
        TimeoutX,
        DelayTest,
        TimeoutTest,
    }
    use RuntimeTimer as T;
    impl timer_stream::Timing for RuntimeTimer {
        fn get_duration(&self) -> std::time::Duration {
            match self {
                T::TimeoutX => std::time::Duration::from_millis(1000u64),
                T::DelayTest => std::time::Duration::from_millis(10u64),
                T::TimeoutTest => std::time::Duration::from_millis(3000u64),
            }
        }
        fn do_reset(&self) -> bool {
            match self {
                T::TimeoutX => true,
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
            let RuntimeInit { reset } = init_vals;
            runtime
                .test
                .handle_init(_grust_reserved_init_instant, reset)
                .await?;
            while let Some(input) = input.next().await {
                match input {
                    I::Clock(clock, _grust_reserved_instant) => {
                        runtime
                            .test
                            .handle_clock(_grust_reserved_instant, clock)
                            .await?;
                    }
                    I::Timer(T::TimeoutX, _grust_reserved_instant) => {
                        runtime
                            .test
                            .handle_timeout_x(_grust_reserved_instant)
                            .await?;
                    }
                    I::Reset(reset, _grust_reserved_instant) => {
                        runtime
                            .test
                            .handle_reset(_grust_reserved_instant, reset)
                            .await?;
                    }
                    I::Timer(T::TimeoutTest, _grust_reserved_instant) => {
                        runtime
                            .test
                            .handle_timeout_test(_grust_reserved_instant)
                            .await?;
                    }
                    I::Timer(T::DelayTest, _grust_reserved_instant) => {
                        runtime
                            .test
                            .handle_delay_test(_grust_reserved_instant)
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
            pub struct O2(i64, bool);
            impl O2 {
                pub fn set(&mut self, o2: i64) {
                    self.1 = self.0 != o2;
                    self.0 = o2;
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
            pub struct Reset(bool, bool);
            impl Reset {
                pub fn set(&mut self, reset: bool) {
                    self.1 = self.0 != reset;
                    self.0 = reset;
                }
                pub fn get(&self) -> bool {
                    self.0
                }
                pub fn take(&mut self) -> bool {
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
            pub struct O1(i64, bool);
            impl O1 {
                pub fn set(&mut self, o1: i64) {
                    self.1 = self.0 != o1;
                    self.0 = o1;
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
            pub o2: ctx_ty::O2,
            pub reset: ctx_ty::Reset,
            pub o1: ctx_ty::O1,
        }
        impl Context {
            fn init() -> Context {
                Default::default()
            }
            fn reset(&mut self) {
                self.o2.reset();
                self.reset.reset();
                self.o1.reset();
            }
        }
        #[derive(Default)]
        pub struct TestServiceStore {
            clock: Option<((), std::time::Instant)>,
            timeout_x: Option<((), std::time::Instant)>,
            reset: Option<(bool, std::time::Instant)>,
        }
        impl TestServiceStore {
            pub fn not_empty(&self) -> bool {
                self.clock.is_some() || self.timeout_x.is_some() || self.reset.is_some()
            }
        }
        pub struct TestService {
            begin: std::time::Instant,
            context: Context,
            delayed: bool,
            input_store: TestServiceStore,
            counter: utils::CounterState,
            counter_1: utils::CounterState,
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
                let counter = <utils::CounterState as grust::core::Component>::init();
                let counter_1 = <utils::CounterState as grust::core::Component>::init();
                TestService {
                    begin: std::time::Instant::now(),
                    context,
                    delayed,
                    input_store,
                    counter,
                    counter_1,
                    output,
                    timer,
                }
            }
            pub async fn handle_init(
                &mut self,
                _grust_reserved_instant: std::time::Instant,
                reset: bool,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_service_timeout(_grust_reserved_instant).await?;
                self.context.reset.set(reset);
                let o1 = <utils::CounterState as grust::core::Component>::step(
                    &mut self.counter_1,
                    utils::CounterInput {
                        res: reset,
                        tick: None,
                    },
                );
                self.context.o1.set(o1);
                self.send_timer(T::TimeoutX, _grust_reserved_instant)
                    .await?;
                let o2 = <utils::CounterState as grust::core::Component>::step(
                    &mut self.counter,
                    utils::CounterInput {
                        res: reset,
                        tick: None,
                    },
                );
                self.context.o2.set(o2);
                self.send_output(
                    O::O2(self.context.o2.get(), _grust_reserved_instant),
                    _grust_reserved_instant,
                )
                .await?;
                self.send_output(
                    O::O1(self.context.o1.get(), _grust_reserved_instant),
                    _grust_reserved_instant,
                )
                .await?;
                Ok(())
            }
            pub async fn handle_timeout_test(
                &mut self,
                _timeout_test_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_time_constraints(_timeout_test_instant).await?;
                self.context.reset();
                self.send_output(
                    O::O2(self.context.o2.get(), _timeout_test_instant),
                    _timeout_test_instant,
                )
                .await?;
                self.send_output(
                    O::O1(self.context.o1.get(), _timeout_test_instant),
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
            pub async fn handle_clock(
                &mut self,
                _clock_instant: std::time::Instant,
                clock: (),
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_clock_instant).await?;
                    self.context.reset();
                    let clock_ref = &mut None;
                    *clock_ref = Some(clock);
                    if clock_ref.is_some() {
                        self.send_timer(T::TimeoutX, _clock_instant).await?;
                    }
                    if self.context.reset.is_new() {
                        let o2 = <utils::CounterState as grust::core::Component>::step(
                            &mut self.counter,
                            utils::CounterInput {
                                res: self.context.reset.get(),
                                tick: None,
                            },
                        );
                        self.context.o2.set(o2);
                    }
                    if self.context.o2.is_new() {
                        self.send_output(
                            O::O2(self.context.o2.get(), _clock_instant),
                            _clock_instant,
                        )
                        .await?;
                    }
                    if clock_ref.is_some() || self.context.reset.is_new() {
                        let o1 = <utils::CounterState as grust::core::Component>::step(
                            &mut self.counter_1,
                            utils::CounterInput {
                                res: self.context.reset.get(),
                                tick: *clock_ref,
                            },
                        );
                        self.context.o1.set(o1);
                    }
                    if self.context.o1.is_new() {
                        self.send_output(
                            O::O1(self.context.o1.get(), _clock_instant),
                            _clock_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self.input_store.clock.replace((clock, _clock_instant));
                    assert ! (unique . is_none () , "flow `clock` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            pub async fn handle_timeout_x(
                &mut self,
                _timeout_x_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_timeout_x_instant).await?;
                    self.context.reset();
                    let x_ref = &mut None;
                    *x_ref = Some(());
                    self.send_timer(T::TimeoutX, _timeout_x_instant).await?;
                    if x_ref.is_some() || self.context.reset.is_new() {
                        let o2 = <utils::CounterState as grust::core::Component>::step(
                            &mut self.counter,
                            utils::CounterInput {
                                res: self.context.reset.get(),
                                tick: *x_ref,
                            },
                        );
                        self.context.o2.set(o2);
                    }
                    if self.context.o2.is_new() {
                        self.send_output(
                            O::O2(self.context.o2.get(), _timeout_x_instant),
                            _timeout_x_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self.input_store.timeout_x.replace(((), _timeout_x_instant));
                    assert ! (unique . is_none () , "flow `timeout_x` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            pub async fn handle_reset(
                &mut self,
                _reset_instant: std::time::Instant,
                reset: bool,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_reset_instant).await?;
                    self.context.reset();
                    self.context.reset.set(reset);
                    if self.context.reset.is_new() {
                        let o2 = <utils::CounterState as grust::core::Component>::step(
                            &mut self.counter,
                            utils::CounterInput {
                                res: reset,
                                tick: None,
                            },
                        );
                        self.context.o2.set(o2);
                    }
                    if self.context.o2.is_new() {
                        self.send_output(
                            O::O2(self.context.o2.get(), _reset_instant),
                            _reset_instant,
                        )
                        .await?;
                    }
                    if self.context.reset.is_new() {
                        let o1 = <utils::CounterState as grust::core::Component>::step(
                            &mut self.counter_1,
                            utils::CounterInput {
                                res: reset,
                                tick: None,
                            },
                        );
                        self.context.o1.set(o1);
                    }
                    if self.context.o1.is_new() {
                        self.send_output(
                            O::O1(self.context.o1.get(), _reset_instant),
                            _reset_instant,
                        )
                        .await?;
                    }
                } else {
                    let unique = self.input_store.reset.replace((reset, _reset_instant));
                    assert ! (unique . is_none () , "flow `reset` changes twice within one minimal delay of the service, consider reducing this delay");
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
                        self.input_store.clock.take(),
                        self.input_store.timeout_x.take(),
                        self.input_store.reset.take(),
                    ) {
                        (None, None, None) => {}
                        (Some((clock, _clock_instant)), None, None) => {
                            let clock_ref = &mut None;
                            *clock_ref = Some(clock);
                            if clock_ref.is_some() {
                                self.send_timer(T::TimeoutX, _clock_instant).await?;
                            }
                            if self.context.reset.is_new() {
                                let o2 = <utils::CounterState as grust::core::Component>::step(
                                    &mut self.counter,
                                    utils::CounterInput {
                                        res: self.context.reset.get(),
                                        tick: None,
                                    },
                                );
                                self.context.o2.set(o2);
                            }
                            if self.context.o2.is_new() {
                                self.send_output(
                                    O::O2(self.context.o2.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            if clock_ref.is_some() || self.context.reset.is_new() {
                                let o1 = <utils::CounterState as grust::core::Component>::step(
                                    &mut self.counter_1,
                                    utils::CounterInput {
                                        res: self.context.reset.get(),
                                        tick: *clock_ref,
                                    },
                                );
                                self.context.o1.set(o1);
                            }
                            if self.context.o1.is_new() {
                                self.send_output(
                                    O::O1(self.context.o1.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (None, Some(((), _timeout_x_instant)), None) => {
                            let x_ref = &mut None;
                            *x_ref = Some(());
                            self.send_timer(T::TimeoutX, _timeout_x_instant).await?;
                            if x_ref.is_some() || self.context.reset.is_new() {
                                let o2 = <utils::CounterState as grust::core::Component>::step(
                                    &mut self.counter,
                                    utils::CounterInput {
                                        res: self.context.reset.get(),
                                        tick: *x_ref,
                                    },
                                );
                                self.context.o2.set(o2);
                            }
                            if self.context.o2.is_new() {
                                self.send_output(
                                    O::O2(self.context.o2.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (Some((clock, _clock_instant)), Some(((), _timeout_x_instant)), None) => {
                            let clock_ref = &mut None;
                            let x_ref = &mut None;
                            *clock_ref = Some(clock);
                            if clock_ref.is_some() {
                                self.send_timer(T::TimeoutX, _clock_instant).await?;
                            } else {
                                *x_ref = Some(());
                                self.send_timer(T::TimeoutX, _clock_instant).await?;
                            }
                            if x_ref.is_some() || self.context.reset.is_new() {
                                let o2 = <utils::CounterState as grust::core::Component>::step(
                                    &mut self.counter,
                                    utils::CounterInput {
                                        res: self.context.reset.get(),
                                        tick: *x_ref,
                                    },
                                );
                                self.context.o2.set(o2);
                            }
                            if self.context.o2.is_new() {
                                self.send_output(
                                    O::O2(self.context.o2.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            if clock_ref.is_some() || self.context.reset.is_new() {
                                let o1 = <utils::CounterState as grust::core::Component>::step(
                                    &mut self.counter_1,
                                    utils::CounterInput {
                                        res: self.context.reset.get(),
                                        tick: *clock_ref,
                                    },
                                );
                                self.context.o1.set(o1);
                            }
                            if self.context.o1.is_new() {
                                self.send_output(
                                    O::O1(self.context.o1.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (None, None, Some((reset, _reset_instant))) => {
                            self.context.reset.set(reset);
                            if self.context.reset.is_new() {
                                let o2 = <utils::CounterState as grust::core::Component>::step(
                                    &mut self.counter,
                                    utils::CounterInput {
                                        res: reset,
                                        tick: None,
                                    },
                                );
                                self.context.o2.set(o2);
                            }
                            if self.context.o2.is_new() {
                                self.send_output(
                                    O::O2(self.context.o2.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            if self.context.reset.is_new() {
                                let o1 = <utils::CounterState as grust::core::Component>::step(
                                    &mut self.counter_1,
                                    utils::CounterInput {
                                        res: reset,
                                        tick: None,
                                    },
                                );
                                self.context.o1.set(o1);
                            }
                            if self.context.o1.is_new() {
                                self.send_output(
                                    O::O1(self.context.o1.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (Some((clock, _clock_instant)), None, Some((reset, _reset_instant))) => {
                            let clock_ref = &mut None;
                            self.context.reset.set(reset);
                            *clock_ref = Some(clock);
                            if clock_ref.is_some() {
                                self.send_timer(T::TimeoutX, _clock_instant).await?;
                            }
                            if self.context.reset.is_new() {
                                let o2 = <utils::CounterState as grust::core::Component>::step(
                                    &mut self.counter,
                                    utils::CounterInput {
                                        res: reset,
                                        tick: None,
                                    },
                                );
                                self.context.o2.set(o2);
                            }
                            if self.context.o2.is_new() {
                                self.send_output(
                                    O::O2(self.context.o2.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            if clock_ref.is_some() || self.context.reset.is_new() {
                                let o1 = <utils::CounterState as grust::core::Component>::step(
                                    &mut self.counter_1,
                                    utils::CounterInput {
                                        res: reset,
                                        tick: *clock_ref,
                                    },
                                );
                                self.context.o1.set(o1);
                            }
                            if self.context.o1.is_new() {
                                self.send_output(
                                    O::O1(self.context.o1.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (None, Some(((), _timeout_x_instant)), Some((reset, _reset_instant))) => {
                            let x_ref = &mut None;
                            self.context.reset.set(reset);
                            if self.context.reset.is_new() {
                                let o1 = <utils::CounterState as grust::core::Component>::step(
                                    &mut self.counter_1,
                                    utils::CounterInput {
                                        res: reset,
                                        tick: None,
                                    },
                                );
                                self.context.o1.set(o1);
                            }
                            if self.context.o1.is_new() {
                                self.send_output(
                                    O::O1(self.context.o1.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            *x_ref = Some(());
                            self.send_timer(T::TimeoutX, _timeout_x_instant).await?;
                            if x_ref.is_some() || self.context.reset.is_new() {
                                let o2 = <utils::CounterState as grust::core::Component>::step(
                                    &mut self.counter,
                                    utils::CounterInput {
                                        res: reset,
                                        tick: *x_ref,
                                    },
                                );
                                self.context.o2.set(o2);
                            }
                            if self.context.o2.is_new() {
                                self.send_output(
                                    O::O2(self.context.o2.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                        }
                        (
                            Some((clock, _clock_instant)),
                            Some(((), _timeout_x_instant)),
                            Some((reset, _reset_instant)),
                        ) => {
                            let clock_ref = &mut None;
                            let x_ref = &mut None;
                            self.context.reset.set(reset);
                            *clock_ref = Some(clock);
                            if clock_ref.is_some() {
                                self.send_timer(T::TimeoutX, _clock_instant).await?;
                            } else {
                                *x_ref = Some(());
                                self.send_timer(T::TimeoutX, _clock_instant).await?;
                            }
                            if x_ref.is_some() || self.context.reset.is_new() {
                                let o2 = <utils::CounterState as grust::core::Component>::step(
                                    &mut self.counter,
                                    utils::CounterInput {
                                        res: reset,
                                        tick: *x_ref,
                                    },
                                );
                                self.context.o2.set(o2);
                            }
                            if self.context.o2.is_new() {
                                self.send_output(
                                    O::O2(self.context.o2.get(), _grust_reserved_instant),
                                    _grust_reserved_instant,
                                )
                                .await?;
                            }
                            if clock_ref.is_some() || self.context.reset.is_new() {
                                let o1 = <utils::CounterState as grust::core::Component>::step(
                                    &mut self.counter_1,
                                    utils::CounterInput {
                                        res: reset,
                                        tick: *clock_ref,
                                    },
                                );
                                self.context.o1.set(o1);
                            }
                            if self.context.o1.is_new() {
                                self.send_output(
                                    O::O1(self.context.o1.get(), _grust_reserved_instant),
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
use futures::{Stream, StreamExt};
pub fn run(
    INIT: std::time::Instant,
    input_stream: impl Stream<Item = runtime::RuntimeInput> + Send + 'static,
    init_signals: runtime::RuntimeInit,
) -> futures::channel::mpsc::Receiver<runtime::RuntimeOutput> {
    const TIMER_CHANNEL_SIZE: usize = 3usize;
    let (timers_sink, timers_stream) = futures::channel::mpsc::channel(TIMER_CHANNEL_SIZE);
    let timers_stream = timers_stream.map(
        |(timer, instant): (runtime::RuntimeTimer, std::time::Instant)| {
            let deadline = instant + timer_stream::Timing::get_duration(&timer);
            runtime::RuntimeInput::Timer(timer, deadline)
        },
    );
    const OUTPUT_CHANNEL_SIZE: usize = 2usize;
    let (output_sink, output_stream) = futures::channel::mpsc::channel(OUTPUT_CHANNEL_SIZE);
    const PRIO_STREAM_SIZE: usize = 100usize;
    let prio_stream = priority_stream::prio_stream::<_, _, PRIO_STREAM_SIZE>(
        futures::stream::select(input_stream, timers_stream),
        runtime::RuntimeInput::order,
    );
    let service = runtime::Runtime::new(output_sink, timers_sink);
    tokio::spawn(service.run_loop(INIT, prio_stream, init_signals));
    output_stream
}
