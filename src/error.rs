use thiserror::Error;

/// Errors produced during command building or validation.
#[derive(Debug, Error, PartialEq)]
pub enum BuildError {
    #[error("option '{0}' conflicts with already-selected option '{1}'")]
    ExclusionViolation(String, String),

    #[error("option '{0}' requires option '{1}' which has not been selected")]
    RequirementViolation(String, String),

    #[error("argument '{0}' conflicts with already-selected argument '{1}'")]
    ArgumentExclusionViolation(String, String),

    #[error("required argument '{0}' was not supplied")]
    MissingRequiredArgument(String),

    #[error("option '{0}' is not valid for the current command node")]
    UnknownOption(String),

    #[error("subcommand '{0}' is not a child of the current command node")]
    UnknownSubCommand(String),

    #[error("format string template '{0}' has unfilled placeholders: {1:?}")]
    UnfilledTemplate(String, Vec<String>),

    #[error("command string is empty")]
    EmptyCommand,
}

/// Errors produced by validation passes over a completed [`FullCommand`](crate::output::FullCommand).
#[derive(Debug, Error, PartialEq)]
pub enum ValidationError {
    #[error("mutually exclusive options present: '{0}' and '{1}'")]
    ConflictingOptions(String, String),

    #[error("option '{0}' present but its required companion '{1}' is missing")]
    MissingRequiredCompanion(String, String),

    #[error("required argument '{0}' is absent from the command string")]
    MissingArgument(String),

    #[error("unknown token '{0}' in command string")]
    UnrecognisedToken(String),
}

/// Errors produced by [`LibraryController`](crate::library::LibraryController) operations.
#[derive(Debug, Error, PartialEq)]
pub enum LibraryError {
    #[error("a CommandMap named '{0}' already exists in this library")]
    DuplicateName(String),

    #[error("no CommandMap named '{0}' found in this library")]
    NotFound(String),

    #[error("serialisation failed: {0}")]
    Serialisation(String),

    #[error("deserialisation failed: {0}")]
    Deserialisation(String),

    #[error("I/O error: {0}")]
    Io(String),
}
