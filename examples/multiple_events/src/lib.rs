#![allow(warnings)]

use grust::grust;
mod macro_output;

grust! {
    #![dump = "examples/multiple_events/src/macro_output.rs"]

    component multiple_events(a: int?, b: int?, v: int) -> (c: int) {
        c = z;
        when {
            (let a = a?, let b = b?) => {
                let z: int = if v > 50 then a else b;
            }
            let a = a? => {
                let z: int = a;
            }
            let b = b? => {
                let z: int = if v > 50 then 0 else b;
            }
            otherwise => {
                let z: int = 5;
            }
        }
    }

    component define_events(a: int?, b: int?, v: int) -> (
        c: int,
        d: float,
        x: int?,
    ) {
        c = z;
        d = when let a = y? then 0.1 otherwise 0.2;
        when {
            (a?, let e = b?) => {
                let z: int =  if v > 50 then e else a;
                let y: unit? = ();
            }
            let _ = a? => {
                let z: int = 2;
                x = 2;
            }
            let _ = b? => {
                let z: int = if v > 50 then 3 else 4;
                x = 2;
            }
            otherwise => {
                let z: int = 0 fby c;
            }
        }
    }

    component final_test(a: int?, b: int?, v: int) -> (
        u: int,
        t: int?,
        x: int?,
    ) {
        t = when a? then a + z;
        u = when (y?, w?) then w + 3 otherwise z + v;

        when {
            (a?, let _ = b?) => {
                let z: int = if v > 50 then 1 else 0;
                let y: unit? = ();
            }
            let a = a? => {
                let z: int = 2;
                x = 2;
            }
            b? => {
                let z: int = if v > 50 then 3 else 4;
                let w: int? = when v > 50 then v + (0 fby u);
                x = 2;
            }
            otherwise => {
                let z: int = 5;
            }
        }
    }
}
