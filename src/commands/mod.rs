
pub mod command_manager;

pub mod command;
pub use command::{Command, UserDecorator};

// ---- src/commands/casual/ ---- //

pub mod casual;

pub use casual::avatar::AvatarCommand;
pub use casual::info::InfoCommand;
pub use casual::nick::NicknameCommand;
pub use casual::verify::VerifyCommand;
pub use casual::about::AboutCommand;
pub use casual::server_info::ServerInfoCommand;
pub use casual::afk::AfkCommand;
pub use casual::poll::PollCommand;
pub use casual::add_emoji::AddEmojiCommand;

// ---- src/commands/moderation/ ---- //

pub mod moderation;

pub use moderation::warn::WarnCommand;
pub use moderation::warnings::WarningsCommand;
pub use moderation::purge::PurgeCommand;
pub use moderation::slowmode::SlowmodeCommand;
pub use moderation::mute::MuteCommand;
pub use moderation::unmute::UnmuteCommand;
pub use moderation::remove_afk::RemoveAfkCommand;
pub use moderation::role::RoleCommand;
pub use moderation::lock::LockCommand;
pub use moderation::unlock::UnlockCommand;
pub use moderation::flag::FlagCommand;
pub use moderation::unflag::UnflagCommand;
pub use moderation::flags::FlagsCommand;
pub use moderation::ban::BanCommand;
pub use moderation::check_ban::CheckBanCommand;
pub use moderation::unban::UnbanCommand;

// ---- src/commands/tickets/ ---- //

#[cfg(feature = "tickets")]
pub mod tickets;

#[cfg(feature = "tickets")]
pub use tickets::open::OpenTicketCommand;
#[cfg(feature = "tickets")]
pub use tickets::close::CloseTicketCommand;
#[cfg(feature = "tickets")]
pub use tickets::claim::ClaimTicketCommand;
#[cfg(feature = "tickets")]
pub use tickets::unclaim::UnclaimTicketCommand;
#[cfg(feature = "tickets")]
pub use tickets::add::AddMemberToTicketCommand;
#[cfg(feature = "tickets")]
pub use tickets::remove::RemoveMemberFromTicketCommand;
