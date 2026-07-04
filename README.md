# Typscord

![Typo the Blue Crab](./docs/banner.svg)

_Typesetting for @everyone._

**Typscord** is a Discord bot that renders [Typst] code.

[Typst]: https://typst.app/

> [!IMPORTANT]
> External third-party packages and fonts are currently unsupported.

## Development Setup

Typscord is written in [Rust] using the [Axum] web framework for the [Tokio] runtime. Compiling and rendering is powered by the official [Typst] library bindings.

[Rust]: https://www.rust-lang.org/
[Axum]: https://docs.rs/axum/latest/axum/
[Tokio]: https://docs.rs/tokio/latest/tokio/

### Loading Environment Variables

> [!IMPORTANT]
> Some environment variables are required only for scripts/automation while others are required at runtime.

```shell
# For Nushell
open .env | from toml | load-env
```

| **Name**                       | **Description**                                                                                                         | Scripts? | Server? |
| ------------------------------ | ----------------------------------------------------------------------------------------------------------------------- | :------: | :-----: |
| `DISCORD_APPLICATION_ID`       | Used for programmatically registering the slash commands via the Discord API.                                           |    ✅    |   ❌    |
| `DISCORD_BOT_TOKEN`            | Used for sending HTTP requests to the Discord API for interaction followup messages.                                    |    ✅    |   ✅    |
| `DISCORD_PUBLIC_KEY`           | Used to verify whether incoming Discord interactions are _actually_ from Discord.                                       |    ❌    |   ✅    |
| `OTEL_EXPORTER_OTLP_ENDPOINT`  | The OpenTelemetry OTLP HTTP endpoint. Docker Compose defaults this to `http://otel:4318`.                               |    ❌    |   ✅    |
| `OTEL_EXPORTER_OTLP_HEADERS`   | Optional comma-separated OpenTelemetry OTLP HTTP headers for authorization or other collector-specific needs.           |    ❌    |   ✅    |
| `PORT`                         | The TCP port to which the network socket will bind. The Docker image defaults this to `3000`.                           |    ❌    |   ✅    |
| `RUST_LOG`                     | The tracing filter. The Docker image defaults this to `typscord=trace`.                                                 |    ❌    |   ✅    |
| `TYPSCORD_COMPILATION_TIMEOUT` | The maximum number of milliseconds to wait for a Typst compilation to finish. The Docker image defaults this to `1000`. |    ❌    |   ✅    |

`OTEL_EXPORTER_OTLP_HEADERS` uses comma-separated key-value pairs. For example:

```shell
OTEL_EXPORTER_OTLP_HEADERS="authorization=Bearer%20TOKEN"
```

### Registering the Slash Commands

The [`discord.json`] contains all of the [application command configurations][bulk-overwrite-global-application-commands] of the bot's supported commands.

[`discord.json`]: ./discord.json
[bulk-overwrite-global-application-commands]: https://discord.com/developers/docs/interactions/application-commands#bulk-overwrite-global-application-commands

```shell
# To register the slash command...
curl --request 'PUT' --header 'Content-Type: application/json' --header "Authorization: Bot $DISCORD_BOT_TOKEN" --data '@discord.json' "https://discord.com/api/v10/applications/$DISCORD_APPLICATION_ID/commands"
```

> [!NOTE]
> See the Nushell script [`register.nu`](./register.nu) for convenience.

### Running the Server

```shell
# Make sure all the environment variables are properly set!
cargo run --release
```

### Running with Docker Compose

Docker Compose starts Typscord and the local `otel-gui` collector. The user must provide `DISCORD_BOT_TOKEN` and `DISCORD_PUBLIC_KEY` in a `.env` at the project root.

```shell
docker compose up --build
```

## Legal

The Typscord project is licensed under the [GNU Affero General Public License v3.0](./LICENSE). However, some files (e.g., brand assets) are exceptions that have been licensed under different terms and limitations. See the [`COPYING.md`] file for more details.

[`COPYING.md`]: ./COPYING.md

## Special Thanks

- [Angelica Raborar][`Anjellyrika`] for designing the original logo and mascot for Typscord: "Typo the Blue Crab".
- [`mattfbacon/typst-bot`] for being an invaluable resource/example of invoking Typst as a library within a Discord bot. Many abstractions in Typscord were inspired by the prior art.

[`Anjellyrika`]: https://github.com/Anjellyrika
[`mattfbacon/typst-bot`]: https://github.com/mattfbacon/typst-bot
