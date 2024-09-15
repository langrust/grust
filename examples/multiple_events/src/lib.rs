#![allow(warnings)]

use grust::grust;
mod macro_output;

grust! {
    #![dump = "examples/multiple_events/src/macro_output.rs"]

    component multiple_events(a: int?, b: int?, v: int) -> (c: int) {
        c = last z;
        let y: unit? = when v > 50 then emit ();
        when {
            (let a = a?, let b = b?) => {
                let aux1: int = a;
                let aux2: int = z;
                let aux3: int = last aux2;
                let z: int = if v > 50 then (last aux1 + aux3) else b;
            }
            let a = a? if a > 0 => {
                let z: int = a;
            }
            (let b = b?, y?) => {
                let z: int = b;
            }
        }
    }

    component define_events(a: int?, b: int?, v: int) -> (
        c: int,
        d: float,
        x: int?,
    ) {
        c = z;
        d = when let a = y? then 0.1;
        when {
            (a?, let e = b?) => {
                let z: int =  if v > 50 then e else a;
                let y: unit? = emit ();
            }
            let _ = a? => {
                let z: int = 2;
                x = emit 2;
            }
            let _ = b? => {
                let z: int = if v > 50 then 3 else 4;
                x = emit 2;
            }
        }
    }

    component final_test(a: int?, b: int?, v: int) -> (
        u: int,
        t: int?,
        x: int?,
    ) {
        t = when a? then emit a + z;
        u = when (y?, w?) then w + 3;
        let test: bool = v > 50;
        let w: int? = when test then emit v + last u;

        when {
            (a?, let _ = b?) => {
                let z: int = if v > 50 then 1 else 0;
                let y: unit? = emit ();
            }
            let a = a? => {
                let z: int = 2;
                x = emit 2;
            }
            b? => {
                let z: int = if v > 50 then 3 else 4;
                x = emit 2;
            }
        }
    }
}
