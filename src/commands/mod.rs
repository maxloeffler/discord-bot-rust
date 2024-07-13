
pub mod command_manager;

pub mod command;
pub use command::Command;

pub mod avatar;
pub mod about;

pub use avatar::AvatarCommand;
pub use about::AboutCommand;
