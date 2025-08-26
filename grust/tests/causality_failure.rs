#![allow(warnings)]
use grust::grust;

grust! {
    #![mode = demo, dump = "grust/out/causality_failure.rs"]

    component match_ok(input: int) -> (sampled: int) {
        match input {
            x if x < 0  => { let mem: int = sampled; sampled = input; }
            _           => { let mem: int = input; sampled = mem; }
        }
    }

    component when_bad(input: int?, ck: float?) -> (sampled: int?) {
        when {
            init    => { mem = 0; },
            input?  => { let mem: int = input; }
            ck?     => { sampled = emit mem; }
        }
    }

    component when_ok(input: int?, ck: float?) -> (sampled: int?) {
        let mem: int = when { init => 0, input? => input };
        sampled = when { ck? => emit mem };
    }
}
