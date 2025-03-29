
# Kalopsian

`Kalopsian` is a 'ready-to-use' strongly opinionated discord bot written entirely in Rust.
It performs various tasks such as ticketing, user moderation, and more.

# Build

To build `Kalopsian` you need to have `Rust` installed on your system.
To compile and afterwards run `Kalopsian` run the following command:

```bash
cargo build --features "full" --release
./target/release/kalopsian

# or

cargo run --features "full"
```

Some functionalities of `Kalopsian` can be disabled in compile time.
To enable only a subset of features, you can replace `full` with any combination of the following features:

| Feature | Description |
| - | - |
| `commands` | The command handler |
| `db_interface` | Interact with the databases from the command line. This can be useful in bootstraping. |
| `message_logs` | Log messages to separate channels in the server |
| `auto_moderation` | Automatically warn, mute, or ban users based on their behavior |
| `tickets` | Manage tickets in the server |

# Setup

## Bot Token

To run `Kalopsian` you need to obtain a bot token from the [Discord Developer Portal](https://discord.com/developers/applications).
Put the token in a file named `token.txt` in the root directory of this project.

## Configuration Parameters

Some of the features of `Kalopsian` require additional configuration.
Whenever `Kalopsian` expects a configuration that is not yet done, it will print an error message to the console.
The configuration is expected to be in `./src/databases/config.db`.
Open it with any SQLite database viewer or use the `db_interface` feature using the following commands:

| Database Interface Command | Description |
| - | - |
| `ls` | List all keys in the current database |
| `cd <database>` | Change the current database to the specified one |
| `get <key ...>` | Get the value(s) of the specified key(s) |
| `get all <key>` | Get all values of the specified key (some values may be lists) |
| `set <key> <value>` | Set the value of the specified key |
| `rm <key>` | Remove the specified key |
| `append <key> <value>` | Append the value to the specified key |

Here is an exhaustive list of all configuration keys used in `Kalopsian` by default:

| Key | Description |
| - | - |
| `command_prefix` | The prefix used to invoke commands, defaults to `?` |
| `guild_main` | The ID of the guild where the bot is operating in |
| `bot_id` | The ID of the bot itself |
| `token` | The bot token used to authenticate with Discord. This is read from `token.txt` |
| `uptime` | The time the bot has been running. This is automatically set by the bot |
| `executed_commands` | The amount of commands executed by the bot. This is automatically updated by the bot |
| `color_primary` | The primary color used in embeds |
| `web_url` | The URL of the server where the bot is running on. This is used to generate links to ticket transcripts |
| `channel_suggestions` | The ID of a channel where suggestions are posted to by the `suggest` command |
| `channel_event_suggestions` | The ID of a channel where event suggestions are posted to by the `suggest` command |
| `channel_transcripts` | The ID of a channel where ticket transcripts are posted to |
| `channel_headmod` | The ID of a channel where the head moderators can discuss witout being logged |
| `channel_reviews` | The ID of a channel where reviews to tickets are posted to by the `review` command |
| `channel_unbanlogs` | The ID of a channel where unban logs are posted to |
| `channel_verify` | The ID of a channel where users can verify themselves before being able to chat |
| `channel_welcome` | The ID of a channel where users are welcomed to the server |
| `channel_tweets` | The ID of a channel where tweets are posted to by the `tweet` command |
| `channel_admin` | The ID of a channel where the administrators can discuss witout being logged |
| `channel_member_count` | The ID of a channel whiches name is updated to the amount of members in the server |
| `channel_protected_log` | The ID of a channel where the bot does **not** log messages |
| `channel_messagelogs` | The ID of a channel where the bot logs edited and deleted messages |
| `channel_muted` | The ID of a channel where muted users can be informed about their status |
| `channel_modlogs` | The ID of a channel where moderation logs are posted to |
| `channel_tickets` | The ID of a channel where users can create tickets |
| `category_music` | The ID of a category where music links can be posted without being detected as external links and deleted |
| `category_tickets` | The ID of a category where tickets are created |
| `category_lockdown` | The ID(s) of categories that are affected by the `lockdown` command |
| `category_protected_slowmode` | The ID(s) of categories that are not affected by the `slowmode` command |
| `category_protected_purge` | The ID(s) of categories that can not be affected by the `purge` command |

## Roles

The following roles are expected to be present in the server for `Kalopsian` to work properly.
Feel free to rename roles but make sure to update all occurrences in the code.

| Role | Description |
| - | - |
| `Trial Moderator` | New moderators that still need to prove themselves |
| `Moderator` | Moderators |
| `Head Moderator` | Head moderators that have more permissions than regular moderators |
| `Administrator` | Administrators that have full bot permissions |
| `Muted` | Users that are muted by the bot because of their behavior |
| `Booster` | Given to users that have boosted the server for additional permissions |
| `Level 10+` | Given to users that have reached level 10 in the server by Mee6 for additional permissions |
| `Level 30+` | Given to users that have reached level 30 in the server by Mee6 for additional permissions |
| `Top Kalie` | Top chatters in the server for additional permissions |
| `Dead Chat` | Given to users who want to be notified when the chat is dead |
| `Tweets` | Given to users who want to be notified when a tweet is posted |
| `Auto Mute` | Given to moderators that want to be notified when a user is automatically muted |
| `User Restrictions` | Users that joined the server but have not yet verified themselves yet |
| `Kalopsian` | Users that have verified themselves |

