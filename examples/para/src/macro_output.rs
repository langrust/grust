use grust::grust_std::rising_edge::{RisingEdgeInput, RisingEdgeState};
pub struct C1Input {
    pub e0: Option<i64>,
}
pub struct C1State {
    mem: i64,
    rising_edge: RisingEdgeState,
}
impl C1State {
    pub fn init() -> C1State {
        C1State {
            mem: 0i64,
            rising_edge: RisingEdgeState::init(),
        }
    }
    pub fn step(&mut self, input: C1Input) -> (i64, Option<i64>) {
        let prev_s2 = self.mem;
        let (s2, e1) = match (input.e0) {
            (Some(e0)) => {
                let s2 = e0;
                let x = e0 > s2;
                let comp_app_rising_edge = self.rising_edge.step(RisingEdgeInput { test: x });
                let e1 = match () {
                    () if comp_app_rising_edge => Some(e0 / e0 - s2),
                    _ => None,
                };
                (s2, e1)
            }
            (_) => {
                let s2 = prev_s2;
                (s2, None)
            }
        };
        self.mem = s2;
        (s2, e1)
    }
}
pub struct C2Input {
    pub e1: Option<i64>,
}
pub struct C2State {
    mem: i64,
    rising_edge: RisingEdgeState,
}
impl C2State {
    pub fn init() -> C2State {
        C2State {
            mem: 0i64,
            rising_edge: RisingEdgeState::init(),
        }
    }
    pub fn step(&mut self, input: C2Input) -> (i64, Option<i64>) {
        let prev_s3 = self.mem;
        let x = prev_s3 > 0i64;
        let comp_app_rising_edge = self.rising_edge.step(RisingEdgeInput { test: x });
        let (s3, e3) = match (input.e1) {
            (Some(e1)) => {
                let s3 = e1;
                (s3, None)
            }
            (_) if comp_app_rising_edge => {
                let s3 = prev_s3;
                let e3 = Some(prev_s3);
                (s3, e3)
            }
            (_) => {
                let s3 = prev_s3;
                (s3, None)
            }
        };
        self.mem = s3;
        (s3, e3)
    }
}
pub struct C3Input {
    pub s2: i64,
}
pub struct C3State {
    rising_edge: RisingEdgeState,
}
impl C3State {
    pub fn init() -> C3State {
        C3State {
            rising_edge: RisingEdgeState::init(),
        }
    }
    pub fn step(&mut self, input: C3Input) -> Option<i64> {
        let x = input.s2 > 1i64;
        let comp_app_rising_edge = self.rising_edge.step(RisingEdgeInput { test: x });
        let e2 = match () {
            () if comp_app_rising_edge => Some(input.s2),
            _ => None,
        };
        e2
    }
}
pub struct C4Input {
    pub e2: Option<i64>,
}
pub struct C4State {
    mem: i64,
}
impl C4State {
    pub fn init() -> C4State {
        C4State { mem: 0i64 }
    }
    pub fn step(&mut self, input: C4Input) -> i64 {
        let s4 = match (input.e2) {
            (Some(e2)) => e2,
            _ => self.mem,
        };
        self.mem = s4;
        s4
    }
}
pub struct C5Input {
    pub s4: i64,
    pub s3: i64,
    pub e3: Option<i64>,
}
pub struct C5State {
    mem: i64,
    rising_edge: RisingEdgeState,
    rising_edge_1: RisingEdgeState,
}
impl C5State {
    pub fn init() -> C5State {
        C5State {
            mem: 0i64,
            rising_edge: RisingEdgeState::init(),
            rising_edge_1: RisingEdgeState::init(),
        }
    }
    pub fn step(&mut self, input: C5Input) -> i64 {
        let x = input.s4 <= 0i64;
        let comp_app_rising_edge = self.rising_edge.step(RisingEdgeInput { test: x });
        let x_1 = input.s3 >= 0i64;
        let comp_app_rising_edge_1 = self.rising_edge.step(RisingEdgeInput { test: x_1 });
        let prev_o = self.mem;
        let o = match (input.e3) {
            (Some(e3)) => {
                let o = e3;
                o
            }
            (_) if comp_app_rising_edge => {
                let o = prev_o * 2i64;
                o
            }
            (_) if comp_app_rising_edge_1 => {
                let o = input.s3;
                o
            }
            (_) => {
                let o = prev_o;
                o
            }
        };
        self.mem = o;
        o
    }
}
pub mod runtime {
    use super::*;
    use futures::{sink::SinkExt, stream::StreamExt};
    use RuntimeInput as I;
    use RuntimeOutput as O;
    use RuntimeTimer as T;
    #[derive(PartialEq)]
    pub enum RuntimeTimer {
        DelayParaMess,
        TimeoutParaMess,
    }
    impl timer_stream::Timing for RuntimeTimer {
        fn get_duration(&self) -> std::time::Duration {
            match self {
                T::DelayParaMess => std::time::Duration::from_millis(10u64),
                T::TimeoutParaMess => std::time::Duration::from_millis(500u64),
            }
        }
        fn do_reset(&self) -> bool {
            match self {
                T::DelayParaMess => true,
                T::TimeoutParaMess => true,
            }
        }
    }
    pub enum RuntimeInput {
        E0(i64, std::time::Instant),
        Timer(T, std::time::Instant),
    }
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
                (I::E0(this, _), I::E0(other, _)) => this.eq(other),
                (I::Timer(this, _), I::Timer(other, _)) => this.eq(other),
                _ => false,
            }
        }
    }
    impl RuntimeInput {
        pub fn get_instant(&self) -> std::time::Instant {
            match self {
                I::E0(_, instant) => *instant,
                I::Timer(_, instant) => *instant,
            }
        }
        pub fn order(v1: &Self, v2: &Self) -> std::cmp::Ordering {
            v1.get_instant().cmp(&v2.get_instant())
        }
    }
    pub enum RuntimeOutput {
        O1(i64, std::time::Instant),
    }
    pub struct Runtime {
        para_mess: para_mess_service::ParaMessService,
        timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
    }
    impl Runtime {
        pub fn new(
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        ) -> Runtime {
            let para_mess = para_mess_service::ParaMessService::init(output, timer.clone());
            Runtime { para_mess, timer }
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
            init_instant: std::time::Instant,
            input: impl futures::Stream<Item = I>,
        ) -> Result<(), futures::channel::mpsc::SendError> {
            futures::pin_mut!(input);
            let mut runtime = self;
            runtime.send_timer(T::TimeoutParaMess, init_instant).await?;
            while let Some(input) = input.next().await {
                match input {
                    I::E0(e0, instant) => {
                        runtime.para_mess.handle_e0(instant, e0).await?;
                    }
                    I::Timer(T::TimeoutParaMess, instant) => {
                        runtime.para_mess.handle_timeout_para_mess(instant).await?;
                    }
                    I::Timer(T::DelayParaMess, instant) => {
                        runtime.para_mess.handle_delay_para_mess(instant).await?;
                    }
                }
            }
            Ok(())
        }
    }
    pub mod para_mess_service {
        use super::*;
        use futures::{sink::SinkExt, stream::StreamExt};
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct S2(i64, bool);
        impl S2 {
            fn set(&mut self, s2: i64) {
                self.0 = s2;
                self.1 = true;
            }
            fn get(&self) -> i64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct E3(i64, bool);
        impl E3 {
            fn set(&mut self, e3: i64) {
                self.0 = e3;
                self.1 = true;
            }
            fn get(&self) -> i64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct E2(i64, bool);
        impl E2 {
            fn set(&mut self, e2: i64) {
                self.0 = e2;
                self.1 = true;
            }
            fn get(&self) -> i64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct E1(i64, bool);
        impl E1 {
            fn set(&mut self, e1: i64) {
                self.0 = e1;
                self.1 = true;
            }
            fn get(&self) -> i64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct S4(i64, bool);
        impl S4 {
            fn set(&mut self, s4: i64) {
                self.0 = s4;
                self.1 = true;
            }
            fn get(&self) -> i64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct O1(i64, bool);
        impl O1 {
            fn set(&mut self, o1: i64) {
                self.0 = o1;
                self.1 = true;
            }
            fn get(&self) -> i64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct S3(i64, bool);
        impl S3 {
            fn set(&mut self, s3: i64) {
                self.0 = s3;
                self.1 = true;
            }
            fn get(&self) -> i64 {
                self.0
            }
            fn is_new(&self) -> bool {
                self.1
            }
            fn reset(&mut self) {
                self.1 = false;
            }
        }
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct Context {
            pub s2: S2,
            pub e3: E3,
            pub e2: E2,
            pub e1: E1,
            pub s4: S4,
            pub o1: O1,
            pub s3: S3,
        }
        impl Context {
            fn init() -> Context {
                Default::default()
            }
            fn reset(&mut self) {
                self.s2.reset();
                self.e3.reset();
                self.e2.reset();
                self.e1.reset();
                self.s4.reset();
                self.o1.reset();
                self.s3.reset();
            }
            fn get_C3_inputs(&self) -> C3Input {
                C3Input { s2: self.s2.get() }
            }
            fn get_C4_inputs(&self, e2: Option<i64>) -> C4Input {
                C4Input { e2: e2 }
            }
            fn get_C1_inputs(&self, e0: Option<i64>) -> C1Input {
                C1Input { e0: e0 }
            }
            fn get_C2_inputs(&self, e1: Option<i64>) -> C2Input {
                C2Input { e1: e1 }
            }
            fn get_C5_inputs(&self, e3: Option<i64>) -> C5Input {
                C5Input {
                    s4: self.s4.get(),
                    s3: self.s3.get(),
                    e3: e3,
                }
            }
        }
        #[derive(Default)]
        pub struct ParaMessServiceStore {
            e0: Option<(i64, std::time::Instant)>,
        }
        impl ParaMessServiceStore {
            pub fn not_empty(&self) -> bool {
                self.e0.is_some()
            }
        }
        pub struct ParaMessService {
            context: Context,
            delayed: bool,
            input_store: ParaMessServiceStore,
            C3: C3State,
            C4: C4State,
            C1: C1State,
            C5: C5State,
            C2: C2State,
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
        }
        impl ParaMessService {
            pub fn init(
                output: futures::channel::mpsc::Sender<O>,
                timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
            ) -> ParaMessService {
                let context = Context::init();
                let delayed = true;
                let input_store = Default::default();
                let C3 = C3State::init();
                let C4 = C4State::init();
                let C1 = C1State::init();
                let C5 = C5State::init();
                let C2 = C2State::init();
                ParaMessService {
                    context,
                    delayed,
                    input_store,
                    C3,
                    C4,
                    C1,
                    C5,
                    C2,
                    output,
                    timer,
                }
            }
            pub async fn handle_delay_para_mess(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.context.reset();
                if self.input_store.not_empty() {
                    self.reset_time_constrains(instant).await?;
                    match (self.input_store.e0.take()) {
                        (None) => {}
                        (Some((e0, e0_instant))) => {
                            let e0_ref = &mut None;
                            let e1_ref = &mut None;
                            let e3_ref = &mut None;
                            let e2_ref = &mut None;
                            *e0_ref = Some(e0);
                            if e0_ref.is_some() {
                                let (s2, e1) = self.C1.step(self.context.get_C1_inputs(*e0_ref));
                                self.context.s2.set(s2);
                                *e1_ref = *e1_ref;
                            }
                            tokio::join!(
                                async {
                                    if e1_ref.is_some() {
                                        let (s3, e3) =
                                            self.C2.step(self.context.get_C2_inputs(*e1_ref));
                                        self.context.s3.set(s3);
                                        *e3_ref = *e3_ref;
                                    }
                                },
                                async {
                                    if self.context.s2.is_new() {
                                        let (e2) = self.C3.step(self.context.get_C3_inputs());
                                        *e2_ref = *e2_ref;
                                    }
                                    if e2_ref.is_some() {
                                        let (s4) =
                                            self.C4.step(self.context.get_C4_inputs(*e2_ref));
                                        self.context.s4.set(s4);
                                    }
                                }
                            );
                            if e3_ref.is_some()
                                || self.context.s4.is_new()
                                || self.context.s3.is_new()
                            {
                                let o1 = self.C5.step(self.context.get_C5_inputs(*e3_ref));
                                self.context.o1.set(o1);
                            }
                            self.send_output(O::O1(self.context.o1.get(), instant))
                                .await?;
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
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer.send((T::DelayParaMess, instant)).await?;
                Ok(())
            }
            pub async fn handle_e0(
                &mut self,
                e0_instant: std::time::Instant,
                e0: i64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constrains(e0_instant).await?;
                    self.context.reset();
                    let e0_ref = &mut None;
                    let e1_ref = &mut None;
                    let e3_ref = &mut None;
                    let e2_ref = &mut None;
                    *e0_ref = Some(e0);
                    if e0_ref.is_some() {
                        let (s2, e1) = self.C1.step(self.context.get_C1_inputs(*e0_ref));
                        self.context.s2.set(s2);
                        *e1_ref = *e1_ref;
                    }
                    tokio::join!(
                        async {
                            if e1_ref.is_some() {
                                let (s3, e3) = self.C2.step(self.context.get_C2_inputs(*e1_ref));
                                self.context.s3.set(s3);
                                *e3_ref = *e3_ref;
                            }
                        },
                        async {
                            if self.context.s2.is_new() {
                                let (e2) = self.C3.step(self.context.get_C3_inputs());
                                *e2_ref = *e2_ref;
                            }
                            if e2_ref.is_some() {
                                let (s4) = self.C4.step(self.context.get_C4_inputs(*e2_ref));
                                self.context.s4.set(s4);
                            }
                        }
                    );
                    if e3_ref.is_some() || self.context.s4.is_new() || self.context.s3.is_new() {
                        let o1 = self.C5.step(self.context.get_C5_inputs(*e3_ref));
                        self.context.o1.set(o1);
                    }
                    self.send_output(O::O1(self.context.o1.get(), e0_instant))
                        .await?;
                } else {
                    let unique = self.input_store.e0.replace((e0, e0_instant));
                    assert!(unique.is_none(), "e0 changes too frequently");
                }
                Ok(())
            }
            pub async fn handle_timeout_para_mess(
                &mut self,
                timeout_para_mess_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_time_constrains(timeout_para_mess_instant)
                    .await?;
                self.context.reset();
                let e3_ref = &mut None;
                let e1_ref = &mut None;
                let e2_ref = &mut None;
                tokio::join!(
                    async {
                        if e1_ref.is_some() {
                            let (s3, e3) = self.C2.step(self.context.get_C2_inputs(*e1_ref));
                            self.context.s3.set(s3);
                            *e3_ref = *e3_ref;
                        }
                    },
                    async {
                        if self.context.s2.is_new() {
                            let (e2) = self.C3.step(self.context.get_C3_inputs());
                            *e2_ref = *e2_ref;
                        }
                        if e2_ref.is_some() {
                            let (s4) = self.C4.step(self.context.get_C4_inputs(*e2_ref));
                            self.context.s4.set(s4);
                        }
                    }
                );
                if e3_ref.is_some() || self.context.s4.is_new() || self.context.s3.is_new() {
                    let o1 = self.C5.step(self.context.get_C5_inputs(*e3_ref));
                    self.context.o1.set(o1);
                }
                self.send_output(O::O1(self.context.o1.get(), timeout_para_mess_instant))
                    .await?;
                Ok(())
            }
            #[inline]
            pub async fn reset_service_timeout(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer.send((T::TimeoutParaMess, instant)).await?;
                Ok(())
            }
            #[inline]
            pub async fn reset_time_constrains(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_service_delay(instant).await?;
                self.reset_service_timeout(instant).await?;
                self.delayed = false;
                Ok(())
            }
            #[inline]
            pub async fn send_output(
                &mut self,
                output: O,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.output.send(output).await?;
                Ok(())
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
        }
    }
}
