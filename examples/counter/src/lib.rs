use grust::grust;

grust! {
    #![dump = "C:/Users/az03049/Documents/gitlab/langrust/grustine/examples/counter/src/macro_output.rs"]

    component counter(res: bool, tick: bool) -> (o: int) {
        o = if res then 0 else (0 fby o) + inc;
        let inc: int = if tick then 1 else 0;
    }

    component test() -> (y: int) {
        y = counter(false fby (y > 35), half).o;
        let half: bool = true fby !half;
    }
}
