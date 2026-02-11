pub mod agent {
    tonic::include_proto!("agent");
}

pub use agent::*;

pub mod runtime;
pub use runtime::*;