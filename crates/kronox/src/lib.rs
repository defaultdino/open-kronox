mod client;
pub mod error;
mod parser;
pub mod schools;

pub use client::Client;
pub use error::Error;
pub use schools::{School, SchoolsConfig};
