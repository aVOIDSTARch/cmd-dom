use serde::{Deserialize, Serialize};
use crate::error::{BuildError, ValidationError};
use crate::output::{CmdFormatString, FullCommand};
use crate::types::{Argument, CommandOption, RootCommand, SubCommand};

// -------------------------------------------------------------------------------------------------
// BuilderState
// -------------------------------------------------------------------------------------------------

/// The accumulating state of one interactive build session.
///
/// The interactive CLI populates this struct step by step as the user
/// answers prompts.  It is then handed to [`CmdBuilder::build`] to produce
/// a [`FullCommand`].
///
/// Keeping state separate from the schema means the schema types stay
/// immutable and `Clone`-able without carrying session noise.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuilderState {
    /// The root command name selected in step 1 of the menu.
    pub root_name: String,

    /// The chain of subcommand names chosen, in order.
    /// e.g. `["repo", "clone"]` for `gh repo clone`.
    pub subcommand_chain: Vec<String>,

    /// Options selected so far.
    /// Each entry is `(option_name, optional_value)`.
    /// `None` value means the option is a pure flag.
    pub selected_options: Vec<(String, Option<String>)>,

    /// Positional arguments supplied so far.
    /// Each entry is `(argument_name, value)`.
    pub selected_arguments: Vec<(String, String)>,
}

impl BuilderState {
    pub fn new(root_name: impl Into<String>) -> Self {
        Self {
            root_name: root_name.into(),
            ..Default::default()
        }
    }

    /// Push a subcommand onto the chain.
    pub fn push_subcommand(&mut self, name: impl Into<String>) {
        self.subcommand_chain.push(name.into());
    }

    /// Record a flag option (no value).
    pub fn add_flag(&mut self, name: impl Into<String>) {
        self.selected_options.push((name.into(), None));
    }

    /// Record an option with a value.
    pub fn add_option(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.selected_options.push((name.into(), Some(value.into())));
    }

    /// Record a positional argument.
    pub fn add_argument(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.selected_arguments.push((name.into(), value.into()));
    }

    /// Returns `true` if `option_name` has already been selected.
    pub fn has_option(&self, option_name: &str) -> bool {
        self.selected_options.iter().any(|(n, _)| n == option_name)
    }

    /// Returns `true` if `arg_name` has already been selected.
    pub fn has_argument(&self, arg_name: &str) -> bool {
        self.selected_arguments.iter().any(|(n, _)| n == arg_name)
    }
}

// -------------------------------------------------------------------------------------------------
// CmdBuilder trait
// -------------------------------------------------------------------------------------------------

/// Builds a [`FullCommand`] from a [`RootCommand`] schema and a [`BuilderState`].
///
/// Implementors are responsible for:
/// - Enforcing `excludes` / `requires` constraints from the schema
/// - Assembling the final shell string in the correct token order
/// - Optionally deriving a [`CmdFormatString`] template from the result
///
/// The trait is object-safe so it can be stored as `Box<dyn CmdBuilder>` if
/// needed by the interactive CLI layer.
pub trait CmdBuilder {
    /// Validate the current [`BuilderState`] against the schema and, if
    /// valid, produce a [`FullCommand`].
    ///
    /// # Errors
    ///
    /// Returns a [`BuildError`] on the first constraint violation encountered.
    /// Call [`CmdBuilder::validate`] first if you want all violations at once.
    fn build(
        &self,
        root: &RootCommand,
        state: &BuilderState,
    ) -> Result<FullCommand, BuildError>;

    /// Run a full validation pass and return every violation found.
    ///
    /// Unlike [`build`](CmdBuilder::build) this does not short-circuit, so
    /// the caller receives the complete list of problems.
    fn validate(
        &self,
        root: &RootCommand,
        state: &BuilderState,
    ) -> Vec<ValidationError>;

    /// Derive a [`CmdFormatString`] template from a finished [`FullCommand`].
    ///
    /// The implementation should replace concrete values in `cmd.cmd_string`
    /// with `{argument_name}` placeholders so the template can be re-used
    /// with different values later.
    fn to_format_string(&self, cmd: &FullCommand, state: &BuilderState) -> CmdFormatString;

    /// Render a saved [`CmdFormatString`] into a [`FullCommand`] by
    /// substituting the provided key-value pairs for placeholders.
    ///
    /// # Errors
    ///
    /// Returns [`BuildError::UnfilledTemplate`] if any placeholder is missing
    /// from `values`.
    fn from_format_string(
        &self,
        fmt: &CmdFormatString,
        values: &[(&str, &str)],
    ) -> Result<FullCommand, BuildError>;
}

// -------------------------------------------------------------------------------------------------
// Constraint helpers — shared logic usable by any CmdBuilder implementation
// -------------------------------------------------------------------------------------------------

/// Check exclusion constraints for a candidate option against an already-built state.
///
/// Returns the first conflict found, or `Ok(())`.
pub fn check_option_exclusions(
    candidate: &CommandOption,
    state: &BuilderState,
) -> Result<(), BuildError> {
    for excluded in &candidate.excludes {
        if state.has_option(excluded) {
            return Err(BuildError::ExclusionViolation(
                candidate.name.clone(),
                excluded.clone(),
            ));
        }
    }
    Ok(())
}

/// Check that all options required by `candidate` are already in state.
pub fn check_option_requirements(
    candidate: &CommandOption,
    state: &BuilderState,
) -> Result<(), BuildError> {
    for required in &candidate.requires {
        if !state.has_option(required) {
            return Err(BuildError::RequirementViolation(
                candidate.name.clone(),
                required.clone(),
            ));
        }
    }
    Ok(())
}

/// Check exclusion constraints for a candidate argument.
pub fn check_argument_exclusions(
    candidate: &Argument,
    state: &BuilderState,
) -> Result<(), BuildError> {
    for excluded in &candidate.excludes {
        if state.has_argument(excluded) {
            return Err(BuildError::ArgumentExclusionViolation(
                candidate.name.clone(),
                excluded.clone(),
            ));
        }
    }
    Ok(())
}

/// Resolve the active [`SubCommand`] from a root using the chain stored in state.
///
/// Returns `None` if the chain is empty (root-level command) or if any
/// name in the chain does not match a child of the current node.
pub fn resolve_subcommand<'a>(
    root: &'a RootCommand,
    state: &BuilderState,
) -> Option<&'a SubCommand> {
    if state.subcommand_chain.is_empty() {
        return None;
    }

    let first = state.subcommand_chain[0].as_str();
    let mut current: &SubCommand = root.subcommands.iter().find(|s| s.name == first)?;

    for segment in &state.subcommand_chain[1..] {
        current = current.subcommands.iter().find(|s| s.name == *segment)?;
    }

    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;


    fn make_option(name: &str, excludes: Vec<&str>, requires: Vec<&str>) -> CommandOption {
        CommandOption {
            name: name.into(),
            description: String::new(),
            help_text: None,
            short: None,
            long: name.into(),
            category: None,
            value: None,
            excludes: excludes.into_iter().map(String::from).collect(),
            requires: requires.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn exclusion_detected() {
        let mut state = BuilderState::new("git");
        state.add_flag("verbose");

        let candidate = make_option("quiet", vec!["verbose"], vec![]);
        assert!(check_option_exclusions(&candidate, &state).is_err());
    }

    #[test]
    fn no_exclusion_when_clear() {
        let state = BuilderState::new("git");
        let candidate = make_option("quiet", vec!["verbose"], vec![]);
        assert!(check_option_exclusions(&candidate, &state).is_ok());
    }

    #[test]
    fn requirement_detected_when_missing() {
        let state = BuilderState::new("git");
        let candidate = make_option("sign-off", vec![], vec!["author"]);
        assert!(check_option_requirements(&candidate, &state).is_err());
    }

    #[test]
    fn requirement_satisfied() {
        let mut state = BuilderState::new("git");
        state.add_flag("author");
        let candidate = make_option("sign-off", vec![], vec!["author"]);
        assert!(check_option_requirements(&candidate, &state).is_ok());
    }
}
