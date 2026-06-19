use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::LibraryError;
use crate::output::CmdFormatString;
use crate::types::RootCommand;

// -------------------------------------------------------------------------------------------------
// Catalog entry — lightweight index
// -------------------------------------------------------------------------------------------------

/// A lightweight index entry for a single command family.
///
/// The catalog lets the interactive CLI present the first-step menu (which
/// root command do you want to build?) without deserializing full
/// [`CommandMap`] trees.  Load the full map only once the user has chosen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmdCatalogEntry {
    /// Must match [`CommandMap::cmd_name`] exactly — used as the lookup key.
    pub name: String,

    /// One-sentence description for the first-step menu.
    pub description: String,
}

// -------------------------------------------------------------------------------------------------
// CommandMap — one per root command family
// -------------------------------------------------------------------------------------------------

/// The complete schema for one root command and everything beneath it.
///
/// `root` is typed [`RootCommand`], which guarantees it can initiate a
/// command string.  The flat `allSubCmds` / `allCmdOptions` arrays from the
/// TypeScript original are deliberately absent — the tree is the single
/// source of truth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMap {
    /// Matches [`CmdCatalogEntry::name`] for O(1) catalog lookup.
    pub cmd_name: String,

    /// Multi-sentence description of what this command family does.
    pub family_description: String,

    /// The root of the command tree.  Being typed `RootCommand` is a
    /// compile-time contract that this node can start a command string.
    pub root: RootCommand,

    /// Reusable templates saved from previous build sessions.
    pub saved_format_strings: Vec<CmdFormatString>,

    /// When this map was first created.
    pub built_on: DateTime<Utc>,

    /// When this map was last modified.
    pub last_updated: DateTime<Utc>,
}

impl CommandMap {
    /// Create a new map with `built_on` and `last_updated` set to now.
    pub fn new(
        cmd_name: impl Into<String>,
        family_description: impl Into<String>,
        root: RootCommand,
    ) -> Self {
        let now = Utc::now();
        Self {
            cmd_name: cmd_name.into(),
            family_description: family_description.into(),
            root,
            saved_format_strings: Vec::new(),
            built_on: now,
            last_updated: now,
        }
    }

    /// Add a format string and update `last_updated`.
    pub fn add_format_string(&mut self, fmt: CmdFormatString) {
        self.saved_format_strings.push(fmt);
        self.last_updated = Utc::now();
    }

    /// Produce a [`CmdCatalogEntry`] from this map without cloning the full tree.
    pub fn catalog_entry(&self) -> CmdCatalogEntry {
        CmdCatalogEntry {
            name: self.cmd_name.clone(),
            description: self.root.description.clone(),
        }
    }
}

// -------------------------------------------------------------------------------------------------
// CommandLibrary
// -------------------------------------------------------------------------------------------------

/// A named collection of [`CommandMap`] objects with a lightweight catalog index.
///
/// The catalog and the command collection are kept in sync by the
/// [`LibraryController`] — do not mutate them directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandLibrary {
    /// Human name for this library, e.g. `"DevOps Toolkit"`.
    pub name: String,

    /// What this library covers.
    pub description: String,

    /// Lightweight index kept in sync with `commands`.
    /// Used to render the first-step menu without loading full trees.
    pub catalog: Vec<CmdCatalogEntry>,

    /// Full command trees.
    pub commands: Vec<CommandMap>,

    /// When this library was first created.
    pub built_on: DateTime<Utc>,

    /// When any map in this library was last modified.
    pub last_updated: DateTime<Utc>,
}

impl CommandLibrary {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            name: name.into(),
            description: description.into(),
            catalog: Vec::new(),
            commands: Vec::new(),
            built_on: now,
            last_updated: now,
        }
    }

    /// Look up a [`CommandMap`] by name without going through the controller.
    pub fn get(&self, name: &str) -> Option<&CommandMap> {
        self.commands.iter().find(|m| m.cmd_name == name)
    }
}

// -------------------------------------------------------------------------------------------------
// LibraryController trait
// -------------------------------------------------------------------------------------------------

/// CRUD and query operations over a [`CommandLibrary`].
///
/// Implementors decide the persistence strategy — in-memory, JSON file,
/// SQLite, etc.  The trait is object-safe.
pub trait LibraryController {
    /// Insert a new [`CommandMap`] and update the catalog.
    ///
    /// # Errors
    ///
    /// [`LibraryError::DuplicateName`] if a map with the same `cmd_name`
    /// already exists.
    fn insert(
        &mut self,
        library: &mut CommandLibrary,
        map: CommandMap,
    ) -> Result<(), LibraryError>;

    /// Replace an existing map by name.
    ///
    /// # Errors
    ///
    /// [`LibraryError::NotFound`] if no map with that name exists.
    fn update(
        &mut self,
        library: &mut CommandLibrary,
        name: &str,
        map: CommandMap,
    ) -> Result<(), LibraryError>;

    /// Remove a map and its catalog entry by name.
    ///
    /// # Errors
    ///
    /// [`LibraryError::NotFound`] if no map with that name exists.
    fn delete(
        &mut self,
        library: &mut CommandLibrary,
        name: &str,
    ) -> Result<(), LibraryError>;

    /// Return the catalog entries — the lightweight index only.
    fn list<'a>(&self, library: &'a CommandLibrary) -> &'a [CmdCatalogEntry];

    /// Retrieve a single [`CommandMap`] by name.
    fn get<'a>(&self, library: &'a CommandLibrary, name: &str) -> Option<&'a CommandMap>;

    /// When was the named map last modified?
    fn last_updated_map(
        &self,
        library: &CommandLibrary,
        name: &str,
    ) -> Option<DateTime<Utc>>;

    /// When was the library itself last modified?
    fn last_updated_library(&self, library: &CommandLibrary) -> DateTime<Utc>;

    /// Serialise the library to a JSON string.
    ///
    /// # Errors
    ///
    /// [`LibraryError::Serialisation`] on failure.
    fn to_json(&self, library: &CommandLibrary) -> Result<String, LibraryError>;

    /// Deserialise a library from a JSON string.
    ///
    /// # Errors
    ///
    /// [`LibraryError::Deserialisation`] on failure.
    fn from_json(&self, json: &str) -> Result<CommandLibrary, LibraryError>;
}

// -------------------------------------------------------------------------------------------------
// InMemoryController — reference implementation
// -------------------------------------------------------------------------------------------------

/// A simple in-memory [`LibraryController`] backed by the `Vec` already
/// inside [`CommandLibrary`].
///
/// Suitable for tests, for an initial CLI prototype, and as the base for a
/// file-backed implementation that wraps this and persists on every write.
#[derive(Debug, Default, Clone)]
pub struct InMemoryController;

impl LibraryController for InMemoryController {
    fn insert(
        &mut self,
        library: &mut CommandLibrary,
        map: CommandMap,
    ) -> Result<(), LibraryError> {
        if library.commands.iter().any(|m| m.cmd_name == map.cmd_name) {
            return Err(LibraryError::DuplicateName(map.cmd_name));
        }
        library.catalog.push(map.catalog_entry());
        library.last_updated = Utc::now();
        library.commands.push(map);
        Ok(())
    }

    fn update(
        &mut self,
        library: &mut CommandLibrary,
        name: &str,
        map: CommandMap,
    ) -> Result<(), LibraryError> {
        let pos = library
            .commands
            .iter()
            .position(|m| m.cmd_name == name)
            .ok_or_else(|| LibraryError::NotFound(name.into()))?;

        // Keep catalog in sync
        if let Some(entry) = library.catalog.iter_mut().find(|e| e.name == name) {
            *entry = map.catalog_entry();
        }

        library.commands[pos] = map;
        library.last_updated = Utc::now();
        Ok(())
    }

    fn delete(
        &mut self,
        library: &mut CommandLibrary,
        name: &str,
    ) -> Result<(), LibraryError> {
        let pos = library
            .commands
            .iter()
            .position(|m| m.cmd_name == name)
            .ok_or_else(|| LibraryError::NotFound(name.into()))?;

        library.commands.remove(pos);
        library.catalog.retain(|e| e.name != name);
        library.last_updated = Utc::now();
        Ok(())
    }

    fn list<'a>(&self, library: &'a CommandLibrary) -> &'a [CmdCatalogEntry] {
        &library.catalog
    }

    fn get<'a>(&self, library: &'a CommandLibrary, name: &str) -> Option<&'a CommandMap> {
        library.commands.iter().find(|m| m.cmd_name == name)
    }

    fn last_updated_map(
        &self,
        library: &CommandLibrary,
        name: &str,
    ) -> Option<DateTime<Utc>> {
        library
            .commands
            .iter()
            .find(|m| m.cmd_name == name)
            .map(|m| m.last_updated)
    }

    fn last_updated_library(&self, library: &CommandLibrary) -> DateTime<Utc> {
        library.last_updated
    }

    fn to_json(&self, library: &CommandLibrary) -> Result<String, LibraryError> {
        serde_json::to_string_pretty(library)
            .map_err(|e| LibraryError::Serialisation(e.to_string()))
    }

    fn from_json(&self, json: &str) -> Result<CommandLibrary, LibraryError> {
        serde_json::from_str(json)
            .map_err(|e| LibraryError::Deserialisation(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AcceptedNext;

    fn bare_root(name: &str) -> RootCommand {
        RootCommand {
            name: name.into(),
            description: format!("{} command", name),
            help_text: None,
            usage_example: None,
            accepts: AcceptedNext::default(),
            subcommands: vec![],
            options: vec![],
            arguments: vec![],
        }
    }

    #[test]
    fn insert_and_list() {
        let mut lib = CommandLibrary::new("test", "test library");
        let mut ctrl = InMemoryController;

        let map = CommandMap::new("git", "Git version control", bare_root("git"));
        ctrl.insert(&mut lib, map).unwrap();

        assert_eq!(ctrl.list(&lib).len(), 1);
        assert_eq!(ctrl.list(&lib)[0].name, "git");
    }

    #[test]
    fn duplicate_insert_is_rejected() {
        let mut lib = CommandLibrary::new("test", "test library");
        let mut ctrl = InMemoryController;

        ctrl.insert(&mut lib, CommandMap::new("git", "Git", bare_root("git"))).unwrap();
        let err = ctrl.insert(&mut lib, CommandMap::new("git", "Git again", bare_root("git")));
        assert!(matches!(err, Err(LibraryError::DuplicateName(_))));
    }

    #[test]
    fn delete_removes_from_catalog_and_commands() {
        let mut lib = CommandLibrary::new("test", "test library");
        let mut ctrl = InMemoryController;

        ctrl.insert(&mut lib, CommandMap::new("git", "Git", bare_root("git"))).unwrap();
        ctrl.delete(&mut lib, "git").unwrap();

        assert!(lib.commands.is_empty());
        assert!(lib.catalog.is_empty());
    }

    #[test]
    fn json_round_trip() {
        let mut lib = CommandLibrary::new("test", "test library");
        let mut ctrl = InMemoryController;

        ctrl.insert(&mut lib, CommandMap::new("git", "Git", bare_root("git"))).unwrap();

        let json = ctrl.to_json(&lib).unwrap();
        let restored = ctrl.from_json(&json).unwrap();

        assert_eq!(restored.commands.len(), 1);
        assert_eq!(restored.commands[0].cmd_name, "git");
    }
}
