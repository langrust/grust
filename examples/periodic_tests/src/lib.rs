#![allow(warnings)]

use grust::grust;

grust! {
    #![dump = "examples/periodic_tests/out/dumped.rs", demo]

    import signal input_s: int;
    import event input_e: int;

    export signal scanned: int;
    export event sampled: int;

    component scan_on(input: int, ck: float?) -> (scanned: int) {
        scanned = when { init => 0, ck? => input };
    }

    component sample_on(input: int?, ck: float?) -> (sampled: int?) {
        let mem: int = when { init => 0, input? => input };
        sampled = when { ck? => emit mem };
    }

    service test @[10, 2000] {
        let event clock: float = period(100);
        scanned = scan_on(input_s, clock);
        sampled = sample_on(input_e, clock);
    }
}
