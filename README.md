# Environment Variables

```shell
# For Nushell
open .env | from toml | load-env
```

| **Name**                 | **Description**                                                                      | Scripts? | Server? |
| ------------------------ | ------------------------------------------------------------------------------------ | :------: | :-----: |
| `DISCORD_APPLICATION_ID` | Used for programmatically registering the slash commands via the Discord API.        |    ✅    |   ❌    |
| `DISCORD_BOT_TOKEN`      | Used for sending HTTP requests to the Discord API for interaction followup messages. |    ❌    |   ✅    |
| `DISCORD_PUBLIC_KEY`     | Used to verify whether incoming Discord interactions are _actually_ from Discord.    |    ❌    |   ✅    |
| `PORT`                   | The TCP port to which the network socket will bind.                                  |    ❌    |   ✅    |

# Registering the Slash Commands

```shell
curl --request 'PUT' --header 'Content-Type: application/json' --header "Authorization: Bot $DISCORD_BOT_TOKEN" --data '@discord.json' "https://discord.com/api/v10/applications/$DISCORD_APPLICATION_ID/commands"
```
