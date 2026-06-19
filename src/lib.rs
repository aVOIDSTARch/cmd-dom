//! cmd_lib — Command schema, builder, and library controller
//!
//! # Module layout
//!
//! - `types`      — Core data types: RootCommand, SubCommand, CommandOption, Argument, etc.
//! - `value`      — ValueKind and OptionValue (split out because they are used everywhere)
//! - `output`     — FullCommand, CmdFormatString (artifacts produced by the builder)
//! - `builder`    — CmdBuilder trait + BuilderState (interactive session state)
//! - `library`    — CommandMap, CommandLibrary, CmdCatalogEntry, LibraryController trait
//! - `error`      — Unified error types for build and library operations

pub mod builder;
pub mod error;
pub mod library;
pub mod output;
pub mod types;
pub mod value;

// Convenience re-exports so consumers can `use cmd_lib::*` for the common types
pub use builder::{BuilderState, CmdBuilder};
pub use error::{BuildError, LibraryError, ValidationError};
pub use library::{CmdCatalogEntry, CommandLibrary, CommandMap, LibraryController};
pub use output::{CmdFormatString, FullCommand};
pub use types::{AcceptedNext, Argument, CommandOption, RootCommand, SubCommand};
pub use value::{OptionValue, ValueKind};
