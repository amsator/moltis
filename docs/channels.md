# Channels

Moltis supports multiple messaging platforms as "channels" — Telegram, Slack,
and Discord. Each channel connects your LLM assistant to users on that platform.

## Supported Channels

| Channel   | Connection Method | Streaming | Notes |
|-----------|-------------------|-----------|-------|
| Telegram  | Long polling      | Edit-in-place | Requires bot token from @BotFather |
| Slack     | Socket Mode       | Edit-in-place | Requires bot + app tokens |
| Discord   | Gateway WebSocket | Edit-in-place | Requires bot token + intents |

## Configuration

Channels can be added via the web UI (**Channels** page → **Connect Channel**)
or in `moltis.toml`:

```toml
[channels.telegram.my-bot]
token = "123456:ABC-DEF..."
dm_policy = "open"
mention_mode = "mention"

[channels.slack.my-workspace]
bot_token = "xoxb-..."
app_token = "xapp-..."
dm_policy = "open"
channel_policy = "open"
activation_mode = "mention"

[channels.discord.my-server]
token = "..."
dm_policy = "open"
guild_policy = "open"
mention_mode = "mention"
```

## Feature Flags

Channel support is compiled via Cargo features (all enabled by default):

```toml
[features]
channel-telegram = ["dep:moltis-telegram"]
channel-discord  = ["dep:moltis-discord"]
channel-slack    = ["dep:moltis-slack"]
```

Build without a channel: `cargo build --no-default-features --features "..."`

---

## Telegram

### Setup

1. Open [@BotFather](https://t.me/BotFather) in Telegram
2. Send `/newbot` and follow the prompts
3. Copy the bot token (looks like `123456:ABC-DEF...`)
4. Add via web UI or config file

### Configuration Options

| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `token` | string | required | Bot token from @BotFather |
| `dm_policy` | `open`, `allowlist`, `disabled` | `open` | Who can DM the bot |
| `mention_mode` | `mention`, `always`, `none` | `mention` | When to respond in groups |
| `allowlist` | array of usernames | `[]` | Users allowed when policy is `allowlist` |
| `model` | string | server default | Override default model for this bot |

### How It Works

The Telegram plugin uses long polling (`getUpdates`) to receive messages. When
a message arrives:

1. Check if it's a DM or group message
2. Apply access control (DM policy, mention detection, allowlist)
3. Log the message
4. Dispatch to the LLM session
5. Stream the response back, editing the message in place

---

## Slack

### Setup

1. Create a new app at [api.slack.com/apps](https://api.slack.com/apps)
2. Enable **Socket Mode** and generate an App-Level Token with `connections:write`
3. Add **Bot Token Scopes**: `chat:write`, `im:write`, `channels:history`,
   `im:history`, `users:read`, `reactions:read`, `app_mentions:read`
4. Subscribe to **Events**: `message.im`, `app_mention`, `reaction_added`
5. Install to workspace
6. Copy the Bot Token (`xoxb-...`) and App Token (`xapp-...`)

See the [OpenClaw Slack docs](https://docs.openclaw.ai/channels/slack) for
detailed instructions.

### Configuration Options

| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `bot_token` | string | required | Bot User OAuth Token (`xoxb-...`) |
| `app_token` | string | required | App-Level Token (`xapp-...`) for Socket Mode |
| `dm_policy` | `open`, `allowlist`, `disabled` | `open` | Who can DM the bot |
| `channel_policy` | `open`, `allowlist`, `disabled` | `open` | Which channels the bot responds in |
| `activation_mode` | `mention`, `always`, `thread_only` | `mention` | When to respond |
| `user_allowlist` | array of Slack user IDs | `[]` | Users allowed when policy is `allowlist` |
| `channel_allowlist` | array of channel IDs | `[]` | Channels allowed when policy is `allowlist` |
| `thread_replies` | bool | `true` | Reply in threads |
| `edit_throttle_ms` | number | `500` | Min interval between message edits |
| `model` | string | server default | Override default model |

### How It Works

The Slack plugin connects via Socket Mode WebSocket. When a message arrives:

1. Parse the event (message, app_mention, etc.)
2. Check channel type (DM, channel, thread)
3. Apply access control (policies, allowlists, activation mode)
4. Strip bot mentions from the message
5. Dispatch to the LLM session
6. Stream response via message edits (throttled to avoid rate limits)

### Activation Modes

- **mention**: Only respond when @mentioned
- **always**: Respond to all messages in allowed channels
- **thread_only**: Respond in threads the bot started or was mentioned in

---

## Discord

### Setup

1. Create an application at [Discord Developer Portal](https://discord.com/developers/applications)
2. Go to **Bot** section, create a bot and copy the token
3. Enable **MESSAGE CONTENT INTENT** and **SERVER MEMBERS INTENT**
4. Go to **OAuth2 > URL Generator**, select `bot` + `applications.commands` scopes
5. Select permissions: View Channels, Send Messages, Read Message History, Embed Links
6. Use the generated URL to invite the bot to your server

See the [OpenClaw Discord docs](https://docs.openclaw.ai/channels/discord) for
detailed instructions.

### Configuration Options

| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `token` | string | required | Bot token from Developer Portal |
| `dm_policy` | `open`, `allowlist`, `disabled` | `open` | Who can DM the bot |
| `guild_policy` | `open`, `allowlist`, `disabled` | `open` | Which servers the bot responds in |
| `mention_mode` | `mention`, `always`, `none` | `mention` | When to respond in servers |
| `user_allowlist` | array of Discord user IDs | `[]` | Users allowed when policy is `allowlist` |
| `guild_allowlist` | array of guild IDs | `[]` | Servers allowed when policy is `allowlist` |
| `channel_allowlist` | array of channel IDs | `[]` | Channels allowed |
| `role_allowlist` | array of role IDs | `[]` | Roles whose members are allowed |
| `use_embeds` | bool | `true` | Use Discord embeds for responses |
| `edit_throttle_ms` | number | `500` | Min interval between message edits |
| `model` | string | server default | Override default model |

### How It Works

The Discord plugin connects via Discord Gateway WebSocket. When a message arrives:

1. Check if from a bot (ignore)
2. Determine context (DM vs guild channel)
3. Apply access control (policies, allowlists, mention detection)
4. Dispatch to the LLM session
5. Stream response via message edits
6. Handle Discord's 2000 character limit by splitting long messages

### Gateway Intents

The bot requests these intents:
- `GUILD_MESSAGES` — receive messages in servers
- `DIRECT_MESSAGES` — receive DMs
- `MESSAGE_CONTENT` — read message text (privileged, must enable in portal)
- `GUILDS` — server membership info

---

## Access Control

All channels share similar access control concepts:

### DM Policy

Controls who can send direct messages to the bot:

- **open**: Anyone can DM
- **allowlist**: Only users in the allowlist
- **disabled**: DMs are ignored

### Channel/Guild Policy

Controls which channels or servers the bot responds in:

- **open**: Respond in any channel the bot is in
- **allowlist**: Only in channels/servers on the allowlist
- **disabled**: Don't respond in channels (DMs only)

### Activation/Mention Mode

Controls when the bot responds in group contexts:

- **mention**: Must @mention the bot
- **always**: Respond to every message
- **thread_only** (Slack): Only in threads bot started or was mentioned in
- **none** (Discord): Don't respond in servers at all

---

## Streaming Responses

All channels support streaming LLM responses via edit-in-place:

1. Bot sends initial placeholder message ("...")
2. As tokens arrive, bot edits the message with accumulated text
3. Edits are throttled (default 500ms) to avoid rate limits
4. Final edit contains complete response

Configure throttle per channel:

```toml
[channels.slack.my-workspace]
edit_throttle_ms = 300  # faster updates
```

---

## Senders Management

The web UI **Senders** tab shows all users who have messaged the bot:

- View message counts and last seen timestamps
- Approve or deny users (updates the allowlist)
- Filter by channel account

This helps manage allowlists without editing config files.
