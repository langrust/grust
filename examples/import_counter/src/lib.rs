#![allow(warnings)]

use grust::grust;

mod utils {
    use crate::grust;

    grust! {
        component counter(res: bool, tick: unit?) -> (o: int) {
            let aux: int = when {
                init => 0,
                tick? => if res then 0 else 1 + last aux,
            };
            o = if res then 0 else aux;
        }
    }
}

grust! {
    use component utils::counter(res: bool, tick: unit?) -> (o: int);

    component test(tick: unit?) -> (y: int) {
        init stop = false;
        let stop: bool = y > 35;
        y = counter(last stop, tick);
    }
}
