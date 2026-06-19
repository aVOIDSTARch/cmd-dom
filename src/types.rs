use serde::{Deserialize, Serialize};
use crate::value::{OptionValue, ValueKind};

// -------------------------------------------------------------------------------------------------
// AcceptedNext
// -------------------------------------------------------------------------------------------------

/// Declares what a command node will accept at the next token position.
///
/// The interactive CLI reads this to decide which prompt to show after the
/// user has confirmed the current node — without having to inspect the full
/// child collections themselves.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AcceptedNext {
    /// This node may be followed by one of its subcommands.
    pub subcommands: bool,
    /// This node may be followed by one or more options.
    pub options: bool,
    /// This node may be followed by one or more positional arguments.
    pub arguments: bool,
}

// -------------------------------------------------------------------------------------------------
// RootCommand
// -------------------------------------------------------------------------------------------------

/// A command that can **initiate** a shell command string.
///
/// Only `RootCommand` values appear at the top level of a [`CommandMap`](crate::library::CommandMap)
/// and are shown in the first step of the interactive menu.  You cannot
/// place a `RootCommand` inside another command's `subcommands` — the
/// type system enforces this by making `subcommands: Vec<SubCommand>`.
///
/// Examples: `git`, `gh`, `docker`, `cargo`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCommand {
    /// The literal binary name, e.g. `"git"`.
    pub name: String,

    /// One-sentence description shown in the library catalog.
    pub description: String,

    /// Extended help shown inside the interactive builder.
    pub help_text: Option<String>,

    /// A representative usage string, e.g. `"git commit -m <message>"`.
    pub usage_example: Option<String>,

    /// What token types are valid immediately after this command.
    pub accepts: AcceptedNext,

    /// Sub-commands this root accepts.  Typed as `SubCommand` — a
    /// `RootCommand` can never be nested here.
    pub subcommands: Vec<SubCommand>,

    /// Options valid at the root level (before any subcommand).
    pub options: Vec<CommandOption>,

    /// Positional arguments valid at the root level.
    pub arguments: Vec<Argument>,
}

// -------------------------------------------------------------------------------------------------
// SubCommand
// -------------------------------------------------------------------------------------------------

/// A command segment that is only meaningful after a parent command.
///
/// `SubCommand` nodes form the interior and leaf nodes of the command tree.
/// They can only appear inside `RootCommand::subcommands` or inside another
/// `SubCommand::subcommands` — never at the top of a [`CommandMap`](crate::library::CommandMap).
///
/// Examples: `repo` (after `gh`), `commit` (after `git`), `run` (after `docker`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubCommand {
    /// The literal segment name, e.g. `"commit"`.
    pub name: String,

    /// One-sentence description shown in the sub-command selection prompt.
    pub description: String,

    /// Extended help shown inside the interactive builder.
    pub help_text: Option<String>,

    /// A representative usage string.
    pub usage_example: Option<String>,

    /// What token types are valid immediately after this sub-command.
    pub accepts: AcceptedNext,

    /// Further sub-commands this node accepts.  Still typed `SubCommand` —
    /// the tree never allows a root to appear as a child.
    pub subcommands: Vec<SubCommand>,

    /// Options specific to this sub-command.
    pub options: Vec<CommandOption>,

    /// Positional arguments for this sub-command.
    pub arguments: Vec<Argument>,
}

// -------------------------------------------------------------------------------------------------
// CommandOption
// -------------------------------------------------------------------------------------------------

/// A flag or option accepted by a command node, e.g. `--message` / `-m`.
///
/// At least one of `short` or `long` must be `Some`.  `long` is used as
/// the canonical identifier in `excludes` and `requires` lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOption {
    /// Canonical identifier used in `excludes` and `requires` on sibling
    /// options.  Typically matches `long` without the `--` prefix.
    pub name: String,

    /// One-sentence description shown in the option selection prompt.
    pub description: String,

    /// Extended help text.
    pub help_text: Option<String>,

    /// Short single-character flag, e.g. `'m'` for `-m`.
    pub short: Option<char>,

    /// Long flag name **without** `--`, e.g. `"message"` for `--message`.
    /// This doubles as the canonical name for constraint resolution.
    pub long: String,

    /// Optional grouping label used to cluster related options in the
    /// interactive prompt, e.g. `"output"`, `"auth"`, `"networking"`.
    pub category: Option<String>,

    /// `None` → pure flag (`--verbose`).
    /// `Some` → the option accepts a value of the described kind.
    pub value: Option<OptionValue>,

    /// `name` values of sibling options that **cannot** be used together
    /// with this one.  Checked during interactive building and validation.
    pub excludes: Vec<String>,

    /// `name` values of sibling options that **must** also be present when
    /// this option is selected.
    pub requires: Vec<String>,
}

// -------------------------------------------------------------------------------------------------
// Argument
// -------------------------------------------------------------------------------------------------

/// A positional argument accepted by a command node.
///
/// Arguments are ordered — their position in the parent `arguments` Vec
/// implies their positional order in the final command string.
/// Set `variadic: true` on the last argument to allow it to consume all
/// remaining tokens (the `file...` pattern).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argument {
    /// Canonical identifier, used in `excludes` on sibling arguments and
    /// as the placeholder label in prompts, e.g. `"file"`, `"remote"`.
    pub name: String,

    /// One-sentence description shown in the argument prompt.
    pub description: String,

    /// Extended help text.
    pub help_text: Option<String>,

    /// Whether the argument must be supplied.
    pub required: bool,

    /// When `true` this argument consumes all remaining tokens.
    /// Only meaningful on the last argument in the list.
    pub variadic: bool,

    /// `None` → the argument is a bare positional with no type constraint.
    /// `Some(ValueKind::Enum(...))` → drives a select-list prompt.
    pub value: Option<ValueKind>,

    /// Default value used when the argument is optional and omitted.
    pub default: Option<String>,

    /// `name` values of sibling arguments that cannot appear alongside
    /// this one.
    pub excludes: Vec<String>,
}
