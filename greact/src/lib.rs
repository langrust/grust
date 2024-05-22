//! # GReact
//!
//! GReact is a library for FRP like operations on data flows in GRRust.
//!
//! There are two interface compilation and execution possible:
//!
//! - **Synchronous:** a sampling rate (base clock) calls for output and the system computes in a **Pull** manner.
//! - **Asynchronous:** events and signals inform their inputs and the system computes in a **Push** manner.
//!
//! ## Data flows
//!
//! We distinguish two kinds of data flows.
//!
//! An **event** is a flow of one-off data occurring at any time.
//! The [Event](event::Event) trait implements its synchronous and asynchronous behaviors.
//!
//! A **signal** is a continuous value varying over time.
//! The [Signal](signal::Signal) trait implements its synchronous and asynchronous behaviors.
//!
//! ## Synchronous interface
//!
//! The synchronous interface is Pull-based, values must be provided when requested.
//! The sampling rate should be faster than events and signal updates, it is the *synchronous principle*
//! (otherwise the system is not Real-Time).
//!
//! ### Synchronous events
//!
//! Event's potential value must be provided when requested.
//! The method [pull](event::Event::pull) transforms an [Event](event::Event) into its
//! synchronous version (a pull stream of optional values).
//!
//! <img src="../../../data/images/sync_sample_event.png" alt="image" width="700" height="auto">
//!
//! ### Synchronous signals
//!
//! Signal's current value must be provided when requested.
//! The method [pull](signal::Signal::pull) transforms a [Signal](signal::Signal) into its
//! synchronous version (a pull stream of values).
//!
//! <img src="../../../data/images/sync_sample_signal.png" alt="image" width="700" height="auto">
//!
//! ## Asynchronous interface
//!
//! The asynchronous interface is Push-based, values must be provided when they occur.
//! We this implementation design, the synchronous principle is satisfied by default.
//!
//! ### Asynchronous events
//!
//! Event's value must be provided when it occurs, a timeout must inform the absence of the event for a period of time.
//! The method [push](event::Event::push), along with a timeout duration, transforms an
//! [Event](event::Event) into its asynchronous version (an asynchronous stream of optional values).
//!
//! <img src="../../../data/images/async_sample_event.png" alt="image" width="700" height="auto">
//!
//! ### Asynchronous signals
//!
//! Signal's update must be provided when it occurs.
//! The method [push](signal::Signal::push) transforms a [Signal](signal::Signal) into its
//! asynchronous version (an asynchronous stream of values).
//!
//! <img src="../../../data/images/async_sample_signal.png" alt="image" width="700" height="auto">
//!
//! ## Operators on events
//!
//! It is possible to perform operations on events.
//! Those operations are not executed on the sampled versions but on the [Event](event::Event) element.
//! The applied operator creates a new event that can be sampled.
//!
//! - [x] `input(Receiver<T>) -> Event<T>`: tranforms a channel receiver into an event.
//!
//! <img src="../../../data/images/input_event.png" alt="image" width="700" height="auto">
//!
//! - [x] `map(Event<T1>, T1 -> T2) -> Event<T2>`: creates an event that maps the given function to the given event.
//!
//! <img src="../../../data/images/map_event.png" alt="image" width="700" height="auto">
//!
//! - [x] `fold(T1, Event<T2>, (T1 x T2) -> T1) -> Event<T1>`: creates an event that folds the given function to the given event.
//!
//! <img src="../../../data/images/fold_event.png" alt="image" width="700" height="auto">
//!
//! - [x] `last_filter(Event<T>, Event<bool>) -> Event<T>`: creates an event that filters the given event with another event.
//!
//! <img src="../../../data/images/last_filter_event.png" alt="image" width="700" height="auto">
//!
//! - [ ] `zip(Event<T1>, Event<T2>) -> Event<(T1 x T2)>`: creates an event that zips the two given events.
//!
//! <img src="../../../data/images/zip_event.png" alt="image" width="700" height="auto">
//!
//! - [ ] `when(Event<T1>, Event<T2>, (T1 x T2) -> T3) -> Event<T3>`: creates an event that zips the two given events and apply a function on them.
//!
//! <img src="../../../data/images/when_event.png" alt="image" width="700" height="auto">
//!
//! - [x] `merge(Event<T>, Event<T>) -> Event<T>`: creates an event that merges the two given events.
//!
//! <img src="../../../data/images/merge_event.png" alt="image" width="700" height="auto">
//!
//! - [ ] `switch(Event<Event<T>>) -> Event<T>`: creates an event that switchs events on an event.
//!
//! ## Operators on signals
//!
//! It is possible to perform operations on signals.
//! Those operations are not executed on the sampled versions but on the [Signal](signal::Signal) element.
//! The applied operator creates a new signal that can be sampled.
//!
//! - [x] `input(T, Receiver<T>) -> Signal<T>`: tranforms a channel receiver into a signal.
//!
//! <img src="../../../data/images/input_signal.png" alt="image" width="700" height="auto">
//!
//! - [x] `map(Signal<T1>, T1 -> T2) -> Signal<T2>`: creates a signal that maps the given function to the given signal.
//!
//! <img src="../../../data/images/map_signal.png" alt="image" width="700" height="auto">
//!
//! - [x] `fold(T1, Signal<T2>, (T1 x T2) -> T1) -> Signal<T1>`: creates a signal that folds the given function to the given signal.
//!
//! <img src="../../../data/images/fold_signal.png" alt="image" width="700" height="auto">
//!
//! - [x] `last_filter(Signal<T>, Signal<bool>) -> Signal<T>`: creates a signal that filters the given signal with another signal.
//!
//! <img src="../../../data/images/last_filter_signal.png" alt="image" width="700" height="auto">
//!
//! - [x] `zip(Signal<T1>, Signal<T2>) -> Signal<(T1 x T2)>`: creates a signal that zips the two given signals.
//!
//! <img src="../../../data/images/zip_signal.png" alt="image" width="700" height="auto">
//!
//! - [ ] `when(Signal<T1>, Signal<T2>, (T1 x T2) -> T3) -> Signal<T3>`: creates a signal that zips the two given signals and apply a function on them.
//!
//! <img src="../../../data/images/when_signal.png" alt="image" width="700" height="auto">
//!
//! - [x] `merge(Signal<T>, Signal<T>) -> Signal<T>`: creates a signal that merges the two given signals.
//!
//! <img src="../../../data/images/merge_signal.png" alt="image" width="700" height="auto">
//!
//! - [ ] `switch(Event<Signal<T>>) -> Signal<T>`: creates a signal that switchs signals on an event.
//!
pub mod event;
pub mod signal;
pub mod stream;
