#![warn(missing_docs)]
//! # LanGRust synchronous language compiler
//! 
//! LanGRust is a [domain-specific language] (DSL) for the automotive industry
//! compiling in Rust, based on [asynchronous event-reactive programming]
//! principles, alongside [synchronous time-reactive] guarantees.
//!
//! Synchronous languages ensure an execution in bounded memory and time.
//! These time-reactive properties make them well-suited for the development
//! of [critical systems]. In the synchronous model, time is a succession
//! of periodical ticks on which the program performs computations.
//!
//! Some event-driven software components (such as the ADAS Sensor Fusion
//! processing) may have to deal with a tremendous number of aperiodic events,
//! which doesn't scale well with legacy synchronous schedulers.
//! The goal is to find the right tradeoff between the efficiency of these
//! asynchronous techniques and the properties required when developing
//! embedded systems (determinism, bounded memory and time, ...).
//!
//! ## Introduction to LanGRust
//! 
//! A LanGRust program is composed of one component and multiple nodes or functions.
//! 
//! Nodes are similar to [Lustre] nodes, they represent a synchronous set of 
//! equations/relations on streams. When executed, nodes perform signals
//! computation at each time step indefinitely. It is possible to create an
//! initialized memory of a stream expression `e` using the `c fby e`
//! (read `c` followed by `e`), where `c` is a constant.
//! This restriction ensures by design that memory is well initialized.
//! 
//! A component is the main element of the program, it represents the system.
//! Its syntax is similar to nodes, in fact it is a node with additional properties.
//! The idea is to synthesise the program in one unique component which is
//! interfaced with sensors/bus/other programs and on which it will be possible to
//! add performance constrains that could be checked at compile time. Components
//! are not completely defined for now in LanGRust, as we do not verify properties
//! thus they are equivalent to nodes.
//! 
//! The last elements of LanGRust are functions. A function is pure calculus on values,
//! it has no side-effects and only returns its result. It is typically used to relieve
//! nodes or components from long computations. By contrast with nodes or components,
//! which endlessly compute signals every time step, functions are executed in one time
//! step and return their result. 
//! 
//! 
//! ## Example
//! ```langrust
//! function integrate(i: float, dx: float, dt: float) -> float {
//!     Di = dx * dt;
//!     return i + Di;
//! }
//! node hard_computation(a: float, v: float, p: float) {
//!     out v_next: float = (v, a, 0.001).map(integrate);
//!     out p_next: float = (p, v, 0.001).map(integrate);
//! }
//! component lazy_system(a: float) {
//!     out v: float = 0.0 fby v_next;
//!     v_next: float = hard_computation(a, v, p).v_next;
//!     out p: float = 0.0 fby p_next;
//!     p_next: float = hard_computation(a, v, p).p_next;
//! }
//! ```
//! 
//! In the example above are illustrated some LanGRust syntactic elements:
//! - output signals are declared with the keyword `out`
//! - `c fby e` creates a buffer of `e` initialized to `c` according to the
//! data-flow appearing bellow
//! - `(e_1, ..., e_k).map(f)` is used to compute the result of a function `f`
//! for each values of streams `e_1`, ..., `e_k` as inputs. The data-flow
//! appearing bellow illustrates its behavior
//! - `n(e_1, ..., e_k).s^o` imports the relations on streams of node `n` that
//! define the output signal `s^o` with `e_1`, ..., `e_k` as node inputs. 
//! 
//! 
//! |expression|t1|t2|t3|t4|
//! |---|---|---|---|---|
//! | `e_1` | `x_1` | `x_2` | `x_3` | `x_4` |
//! | `c fby e_1` | `c`   | `x_1` | `x_2` | `x_3` |
//! | `e_2` | `y_1` | `y_2` | `y_3` | `y_4` |
//! | `(e_1, e_2).map(f)` | `f(x_1, y_1)` | `f(x_2, y_2)` | `f(x_3, y_3)` | `f(x_4, y_4)` |
//! 
//! 
//! [domain-specific language]: https://en.wikipedia.org/wiki/Domain-specific_language
//! [asynchronous event-reactive programming]: https://en.wikipedia.org/wiki/Reactive_programming
//! [synchronous time-reactive]: https://en.wikipedia.org/wiki/Synchronous_programming_language
//! [critical systems]: https://en.wikipedia.org/wiki/Critical_system
//! [Lustre]: https://en.wikipedia.org/wiki/Lustre_(programming_language)

/// Utilitary module.
pub mod util;
/// LanGRust AST module.
pub mod ast;

#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(
    #[doc="Load parser in src/langrust.lalrpop."]
    #[allow(missing_docs)]
    pub langrust
);

extern crate codespan_reporting;