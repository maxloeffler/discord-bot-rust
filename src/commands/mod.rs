
pub mod command_manager;

pub mod command;
pub use command::{Command, UserDecorator};

// ---- src/commands/casual/ ---- //

pub mod casual;

pub use casual::avatar::AvatarCommand;
pub use casual::info::InfoCommand;
pub use casual::nick::NicknameCommand;
pub use casual::verify::VerifyCommand;

// ---- src/commands/moderation/ ---- //

pub mod moderation;

pub use moderation::warn::WarnCommand;

// ---- src/commands/tickets/ ---- //

pub mod tickets;

pub use tickets::open::OpenTicketCommand;
pub use tickets::close::CloseTicketCommand;
pub use tickets::claim::ClaimTicketCommand;
pub use tickets::unclaim::UnclaimTicketCommand;
pub use tickets::add::AddMemberToTicketCommand;
pub use tickets::remove::RemoveMemberFromTicketCommand;
