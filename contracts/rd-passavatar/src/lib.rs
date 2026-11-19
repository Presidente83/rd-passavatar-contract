mod contract;
pub mod msg;
mod queries;
pub mod state;
pub use contract::{execute, instantiate, Error};
pub use queries::query;
#[cfg(test)]
mod tests;
