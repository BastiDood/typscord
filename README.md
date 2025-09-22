# Typscord

A Discord bot for rendering Typst code.

## Loading Environment Variables

```shell
# For Nushell
open .env | from toml | load-env
```

| **Name**                       | **Description**                                                                      | Scripts? | Server? |
| ------------------------------ | ------------------------------------------------------------------------------------ | :------: | :-----: |
| `DISCORD_APPLICATION_ID`       | Used for programmatically registering the slash commands via the Discord API.        |    ✅    |   ❌    |
| `DISCORD_BOT_TOKEN`            | Used for sending HTTP requests to the Discord API for interaction followup messages. |    ❌    |   ✅    |
| `DISCORD_PUBLIC_KEY`           | Used to verify whether incoming Discord interactions are _actually_ from Discord.    |    ❌    |   ✅    |
| `TYPSCORD_COMPILATION_TIMEOUT` | The maximum number of seconds to wait for a Typst compilation to finish.             |    ❌    |   ✅    |
| `PORT`                         | The TCP port to which the network socket will bind.                                  |    ❌    |   ✅    |

## Registering the Slash Commands

```shell
curl --request 'PUT' --header 'Content-Type: application/json' --header "Authorization: Bot $DISCORD_BOT_TOKEN" --data '@discord.json' "https://discord.com/api/v10/applications/$DISCORD_APPLICATION_ID/commands"
```

## Special Thanks

- [`mattfbacon/typst-bot`] for being an invaluable resource/example of invoking Typst as a library within a Discord bot. Many abstractions in Typscord were inspired by the prior art.

[`mattfbacon/typst-bot`]: https://github.com/mattfbacon/typst-bot
