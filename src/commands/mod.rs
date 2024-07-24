
pub mod command_manager;

pub mod command;
pub use command::{Command, UserDecorator};

pub mod avatar;
pub mod info;

pub use avatar::AvatarCommand;
pub use info::InfoCommand;

// ---- src/commands/mod_commands/ ----

pub mod mod_commands;
pub use mod_commands::warn::WarnCommand;

// ---- src/commands/ticket_commands/ ----

pub mod ticket_commands;

pub use ticket_commands::open::OpenTicketCommand;
pub use ticket_commands::claim::ClaimTicketCommand;
