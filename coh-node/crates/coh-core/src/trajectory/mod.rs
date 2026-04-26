pub mod engine;
pub mod types;

pub use engine::TrajectoryEngine;
pub use types::{StateNode, Trajectory, Transition};

#[cfg(test)]
mod equivalence_tests;
