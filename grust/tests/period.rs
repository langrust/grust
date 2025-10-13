#![allow(warnings)]

use grust::grust;

grust! {
    #![dump = "grust/out/period.rs", mode = demo]

    import signal input_s: int;
    import event input_e: int;

    export signal scanned: int;
    export event sampled: int;

    service test @[10, 2000] {
        let event clock: float = period(100);
        scanned = scan_on(input_s, clock);
        sampled = sample_on(input_e, clock);
    }
}
