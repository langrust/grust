pub struct C1Input {
    pub e0: Option<i64>,
}
pub struct C1State {
    last_s2: i64,
}
impl grust::core::Component for C1State {
    type Input = C1Input;
    type Output = (i64, Option<i64>);
    fn init() -> C1State {
        C1State { last_s2: 0i64 }
    }
    fn step(&mut self, input: C1Input) -> (i64, Option<i64>) {
        let prev_s2 = self.last_s2;
        let (s2, e1) = match (input.e0) {
            (Some(e0)) if e0 > prev_s2 => {
                let s2 = e0;
                let e1 = Some(e0 / (e0 - s2));
                (s2, e1)
            }
            (Some(e0)) => {
                let s2 = e0;
                (s2, None)
            }
            (_) => (self.last_s2, None),
        };
        self.last_s2 = s2;
        (s2, e1)
    }
}
pub struct C2Input {
    pub e1: Option<i64>,
}
pub struct C2State {
    last_s3: i64,
    last_x: bool,
}
impl grust::core::Component for C2State {
    type Input = C2Input;
    type Output = (i64, Option<i64>);
    fn init() -> C2State {
        C2State {
            last_s3: 0i64,
            last_x: false,
        }
    }
    fn step(&mut self, input: C2Input) -> (i64, Option<i64>) {
        let prev_s3 = self.last_s3;
        let x = prev_s3 > 0i64;
        let (s3, e3) = match (input.e1) {
            (Some(e1)) => {
                let s3 = e1;
                (s3, None)
            }
            (_) if x && !(self.last_x) => {
                let s3 = prev_s3;
                let e3 = Some(prev_s3);
                (s3, e3)
            }
            (_) => (self.last_s3, None),
        };
        self.last_s3 = s3;
        self.last_x = x;
        (s3, e3)
    }
}
pub struct C3Input {
    pub s2: i64,
}
pub struct C3State {
    last_x: bool,
}
impl grust::core::Component for C3State {
    type Input = C3Input;
    type Output = Option<i64>;
    fn init() -> C3State {
        C3State { last_x: false }
    }
    fn step(&mut self, input: C3Input) -> Option<i64> {
        let x = input.s2 > 1i64;
        let e2 = match () {
            () if x && !(self.last_x) => Some(input.s2),
            () => None,
        };
        self.last_x = x;
        e2
    }
}
pub struct C4Input {
    pub e2: Option<i64>,
}
pub struct C4State {
    last_s4: i64,
}
impl grust::core::Component for C4State {
    type Input = C4Input;
    type Output = i64;
    fn init() -> C4State {
        C4State { last_s4: 0i64 }
    }
    fn step(&mut self, input: C4Input) -> i64 {
        let s4 = match (input.e2) {
            (Some(e2)) => e2,
            (_) => self.last_s4,
        };
        self.last_s4 = s4;
        s4
    }
}
pub struct C5Input {
    pub s4: i64,
    pub s3: i64,
    pub e3: Option<i64>,
}
pub struct C5State {
    last_o: i64,
    last_x: bool,
    last_x_1: bool,
}
impl grust::core::Component for C5State {
    type Input = C5Input;
    type Output = i64;
    fn init() -> C5State {
        C5State {
            last_o: 0i64,
            last_x: false,
            last_x_1: false,
        }
    }
    fn step(&mut self, input: C5Input) -> i64 {
        let x = input.s4 <= 0i64;
        let x_1 = input.s3 >= 0i64;
        let prev_o = self.last_o;
        let o = match (input.e3) {
            (Some(e3)) => {
                let o = e3;
                o
            }
            (_) if x && !(self.last_x) => {
                let o = prev_o * 2i64;
                o
            }
            (_) if x_1 && !(self.last_x_1) => {
                let o = input.s3;
                o
            }
            (_) => self.last_o,
        };
        self.last_o = o;
        self.last_x = x;
        self.last_x_1 = x_1;
        o
    }
}
pub mod runtime {
    use super::*;
    use futures::{sink::SinkExt, stream::StreamExt};
    pub enum RuntimeInput {
        E0(i64, std::time::Instant),
    }
    use RuntimeInput as I;
    impl priority_stream::Reset for RuntimeInput {
        fn do_reset(&self) -> bool {
            match self {
                _ => false,
            }
        }
    }
    impl PartialEq for RuntimeInput {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (I::E0(this, _), I::E0(other, _)) => this.eq(other),
                _ => false,
            }
        }
    }
    impl RuntimeInput {
        pub fn get_instant(&self) -> std::time::Instant {
            match self {
                I::E0(_, _grust_reserved_instant) => *_grust_reserved_instant,
            }
        }
        pub fn order(v1: &Self, v2: &Self) -> std::cmp::Ordering {
            v1.get_instant().cmp(&v2.get_instant())
        }
    }
    #[derive(Debug, PartialEq)]
    pub enum RuntimeOutput {
        O1(i64, std::time::Instant),
    }
    use RuntimeOutput as O;
    pub struct Runtime {
        para_mess: para_mess_service::ParaMessService,
        output: futures::channel::mpsc::Sender<O>,
    }
    impl Runtime {
        pub fn new(output: futures::channel::mpsc::Sender<O>) -> Runtime {
            let para_mess = para_mess_service::ParaMessService::init(output.clone());
            Runtime { para_mess, output }
        }
        pub async fn run_loop(
            self,
            _grust_reserved_init_instant: std::time::Instant,
            input: impl futures::Stream<Item = I>,
        ) -> Result<(), futures::channel::mpsc::SendError> {
            futures::pin_mut!(input);
            let mut runtime = self;
            runtime
                .para_mess
                .handle_init(_grust_reserved_init_instant)
                .await?;
            while let Some(input) = input.next().await {
                match input {
                    I::E0(e0, _grust_reserved_instant) => {
                        runtime
                            .para_mess
                            .handle_e0(_grust_reserved_instant, e0)
                            .await?;
                    }
                }
            }
            Ok(())
        }
    }
    pub mod para_mess_service {
        use super::*;
        use futures::{sink::SinkExt, stream::StreamExt};
        mod ctx_ty {
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct S2(i64, bool);
            impl S2 {
                pub fn set(&mut self, s2: i64) {
                    self.1 = self.0 != s2;
                    self.0 = s2;
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
            pub struct S4(i64, bool);
            impl S4 {
                pub fn set(&mut self, s4: i64) {
                    self.1 = self.0 != s4;
                    self.0 = s4;
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
            pub struct S3(i64, bool);
            impl S3 {
                pub fn set(&mut self, s3: i64) {
                    self.1 = self.0 != s3;
                    self.0 = s3;
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
            pub struct E2(i64, bool);
            impl E2 {
                pub fn set(&mut self, e2: i64) {
                    self.1 = self.0 != e2;
                    self.0 = e2;
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
            pub struct E1(i64, bool);
            impl E1 {
                pub fn set(&mut self, e1: i64) {
                    self.1 = self.0 != e1;
                    self.0 = e1;
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
            pub struct E3(i64, bool);
            impl E3 {
                pub fn set(&mut self, e3: i64) {
                    self.1 = self.0 != e3;
                    self.0 = e3;
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
            pub s2: ctx_ty::S2,
            pub s4: ctx_ty::S4,
            pub s3: ctx_ty::S3,
            pub e2: ctx_ty::E2,
            pub e1: ctx_ty::E1,
            pub e3: ctx_ty::E3,
            pub o1: ctx_ty::O1,
        }
        impl Context {
            fn init() -> Context {
                Default::default()
            }
            fn reset(&mut self) {
                self.s2.reset();
                self.s4.reset();
                self.s3.reset();
                self.e2.reset();
                self.e1.reset();
                self.e3.reset();
                self.o1.reset();
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
            begin: std::time::Instant,
            context: Context,
            delayed: bool,
            input_store: ParaMessServiceStore,
            c_4: C4State,
            c_1: C1State,
            c_5: C5State,
            c_2: C2State,
            c_3: C3State,
            output: futures::channel::mpsc::Sender<O>,
        }
        impl ParaMessService {
            pub fn init(output: futures::channel::mpsc::Sender<O>) -> ParaMessService {
                let context = Context::init();
                let delayed = true;
                let input_store = Default::default();
                let c_4 = <C4State as grust::core::Component>::init();
                let c_1 = <C1State as grust::core::Component>::init();
                let c_5 = <C5State as grust::core::Component>::init();
                let c_2 = <C2State as grust::core::Component>::init();
                let c_3 = <C3State as grust::core::Component>::init();
                ParaMessService {
                    begin: std::time::Instant::now(),
                    context,
                    delayed,
                    input_store,
                    c_4,
                    c_1,
                    c_5,
                    c_2,
                    c_3,
                    output,
                }
            }
            pub async fn handle_init(
                &mut self,
                _grust_reserved_instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                Ok(())
            }
            pub async fn handle_e0(
                &mut self,
                _e0_instant: std::time::Instant,
                e0: i64,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                if self.delayed {
                    self.reset_time_constraints(_e0_instant).await?;
                    self.context.reset();
                    let e0_ref = &mut None;
                    let e3_ref = &mut None;
                    let e1_ref = &mut None;
                    let e2_ref = &mut None;
                    *e0_ref = Some(e0);
                    if e0_ref.is_some() {
                        let (s2, e1) = <C1State as grust::core::Component>::step(
                            &mut self.c_1,
                            C1Input { e0: *e0_ref },
                        );
                        self.context.s2.set(s2);
                        *e1_ref = e1;
                    }
                    tokio::join!(
                        async {
                            if e1_ref.is_some() {
                                let (s3, e3) = <C2State as grust::core::Component>::step(
                                    &mut self.c_2,
                                    C2Input { e1: *e1_ref },
                                );
                                self.context.s3.set(s3);
                                *e3_ref = e3;
                            }
                        },
                        async {
                            if self.context.s2.is_new() {
                                let (e2) = <C3State as grust::core::Component>::step(
                                    &mut self.c_3,
                                    C3Input {
                                        s2: self.context.s2.get(),
                                    },
                                );
                                *e2_ref = e2;
                            }
                            if e2_ref.is_some() {
                                let (s4) = <C4State as grust::core::Component>::step(
                                    &mut self.c_4,
                                    C4Input { e2: *e2_ref },
                                );
                                self.context.s4.set(s4);
                            }
                        }
                    );
                    if e3_ref.is_some() || self.context.s4.is_new() || self.context.s3.is_new() {
                        let o1 = <C5State as grust::core::Component>::step(
                            &mut self.c_5,
                            C5Input {
                                s4: self.context.s4.get(),
                                s3: self.context.s3.get(),
                                e3: *e3_ref,
                            },
                        );
                        self.context.o1.set(o1);
                    }
                    if self.context.o1.is_new() {
                        self.send_output(O::O1(self.context.o1.get(), _e0_instant), _e0_instant)
                            .await?;
                    }
                } else {
                    let unique = self.input_store.e0.replace((e0, _e0_instant));
                    assert ! (unique . is_none () , "flow `e0` changes twice within one minimal delay of the service, consider reducing this delay");
                }
                Ok(())
            }
            #[inline]
            pub async fn reset_time_constraints(
                &mut self,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.delayed = false;
                Ok(())
            }
            #[inline]
            pub async fn send_output(
                &mut self,
                output: O,
                instant: std::time::Instant,
            ) -> Result<(), futures::channel::mpsc::SendError> {
                self.output.feed(output).await?;
                Ok(())
            }
        }
    }
}
