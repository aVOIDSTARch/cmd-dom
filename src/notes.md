// Quick synopsis of created files 

## value.rs 

— pulled out of types.rs because ValueKind and OptionValue are referenced by both CommandOption and Argument. Keeping them separate prevents circular module dependencies when the crate grows.

## types.rs 

— RootCommand and SubCommand are distinct structs with subcommands: Vec<SubCommand> on both. This is the type-system enforcement of your question — a RootCommand can only ever appear as the top of a CommandMap::root field. The compiler refuses to let you nest one as a child.

## output.rs 

— FullCommand and CmdFormatString are separate from the schema types because they’re products of a build session, not schema definitions. CmdFormatString::render() is implemented here with a test covering both the happy path and missing placeholder detection.

## builder.rs 

— CmdBuilder is a trait, not a struct, because the interactive TUI layer will implement it differently than a headless batch builder would. BuilderState accumulates session state separately from the schema so the schema stays immutable. The free functions check_option_exclusions, check_option_requirements, and resolve_subcommand are public so any CmdBuilder implementor can use the same constraint logic without re-implementing it.

## library.rs 

— InMemoryController is the concrete reference implementation of LibraryController. The JSON round-trip is tested. The catalog and command Vec are kept in sync on every write through the controller — direct mutation of library.commands bypasses that invariant, which is why the controller takes &mut CommandLibrary rather than exposing the fields.

## What’s not here yet 

— a concrete CmdBuilder implementation. That’s intentional: it belongs in your TUI crate, not this schema crate. This crate is the data model and contract; the interactive layer is a separate concern.