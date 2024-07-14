
pub mod mod_commands;

pub mod command_manager;

pub mod command;
pub use command::{Command, UserDecorator};

pub mod avatar;

pub use avatar::AvatarCommand;
pub use mod_commands::warn::WarnCommand;
