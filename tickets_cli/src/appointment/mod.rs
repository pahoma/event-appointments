pub mod create;
pub mod delete;
pub mod generate;
pub mod send;

pub use create::*;
pub use delete::*;
pub use generate::*;
pub use send::*;

#[cfg(test)]
use std::sync::Mutex;
#[cfg(test)]
use once_cell::sync::Lazy;
#[cfg(test)]
static ENV_VAR_LOCK_TEST: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
