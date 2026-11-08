# secrets/

Real values for the `keyhere` placeholders. Git ignores every file here except this README.
Code: `Program/Platform/StServers/StsCore/src/secrets.rs`.

## How a service finds a secret

1. Environment variable `STS_SECRET_<NAME>` (name in upper case, other characters → `_`).
2. File `secrets/<name>`, or `$STS_SECRETS_DIR/<name>`.

A config value that is not `keyhere` is used as it is.

## Names

| Name | Replaces | Value |
| --- | --- | --- |
| `db.<Logical>.password` | `Db.ini [Sql.<Logical>] Password` | SQL password of the `bns-mssql` container |
| `auth.token_private.pem` | ❓ original token signing key in `Certificate/*` | Our STS token private key. Its public half is patched into the client |
| `cligate.tls.pem` | CliGate TLS certificate | A new certificate. Do not reuse the token key |
| `route.tls.key` | `Route.ini [TLS] EncryptedKey` | Not needed on one host (TLS off) |
| `game.<App>.secret` | 5th column of `Game.ini [Configurations]` | A new random secret; LobbyDaemon uses the same value |

## Rules

- Set file mode 600 (`chmod 600 secrets/*`). The services warn if others can read a file.
- Do not copy NCSoft keys or passwords here. Generate our own.
- Do not commit, print or paste these files.
