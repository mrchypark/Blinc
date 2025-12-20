//! Blinc Core Runtime
//!
//! This crate provides the foundational primitives for the Blinc UI framework:
//!
//! - **Reactive Signals**: Fine-grained reactivity without VDOM overhead
//! - **State Machines**: Harel statecharts for widget interaction states
//! - **Event Dispatch**: Unified event handling across platforms
//!
//! # Example
//!
//! ```rust
//! use blinc_core::reactive::ReactiveGraph;
//!
//! let mut graph = ReactiveGraph::new();
//!
//! // Create a signal
//! let count = graph.create_signal(0i32);
//!
//! // Create a derived value
//! let doubled = graph.create_derived(move |g| {
//!     g.get(count).unwrap_or(0) * 2
//! });
//!
//! // Create an effect
//! let _effect = graph.create_effect(move |g| {
//!     println!("Count is now: {:?}", g.get(count));
//! });
//!
//! // Update the signal
//! graph.set(count, 5);
//! assert_eq!(graph.get_derived(doubled), Some(10));
//! ```

pub mod events;
pub mod fsm;
pub mod reactive;
pub mod runtime;

pub use events::{Event, EventDispatcher, EventType};
pub use fsm::{FsmId, FsmRuntime, StateId, StateMachine, Transition};
pub use reactive::{Derived, DerivedId, Effect, EffectId, ReactiveGraph, Signal, SignalId};
pub use runtime::BlincRuntime;
