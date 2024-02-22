# Attester Service

The Attester Service is responsible for generating various credentials for users, with an authentication mechanism that distinguishes between regular users and employees. Users can request different types of credentials, which can then be approved by employees. The service implements the KILT [Credential API](https://github.com/KILTprotocol/spec-ext-credential-api), allowing users to store their credentials in their identity wallet. Authentication is facilitated by fetching a JWT token from [OpenDID](https://github.com/KILTprotocol/opendid). Users can log in with a DID, while employees require additional credentials.

A demonstration deployment for Peregrine can be accessed [here](https://dena-attester-dev.kilt.io/#/login), and a Spiritnet deployment is available [here](https://dena-attester.kilt.io/#/login).

## Usage

All environment variables must be configured in a `config.yaml` file. An example `config.yaml` file is provided [here](./config_example.yaml), with explanations of the variables included.

### Local Debugging Frontend

The frontend utilizes Vite as a bundler. To develop, simply run `yarn dev`. To build the frontend, execute `yarn build`.

### Local Debugging Backend VsCode

Create a `.vscode/launch.json` file and paste the following content:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug executable 'attester-backend'",
      "cargo": {
        "args": ["build", "--bins"],
        "filter": {
          "name": "attester_peregrine",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "env": {
        "CONFIG": "./config.yaml"
      }
    },
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug unit tests in executable 'attester-backend'",
      "cargo": {
        "args": [
          "test",
          "--no-run",
          "--bin=attester-backend",
          "--package=attester-backend"
        ],
        "filter": {
          "name": "attester-backend",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

Please note that the above configuration is tailored for Peregrine. If debugging on Spiritnet, adjust the filter accordingly.

### Docker

A docker-compose file is provided. To start the containers, run `docker-compose up`.

### Database

The Rust backend utilizes sqlx for database interactions. If a query is modified, update the metadata to support offline compile-time verification using the command `cargo sqlx prepare`. New migrations can be added with `cargo sqlx migrate add`, and existing migrations can be executed via CLI with `cargo sqlx migrate run`. The source code manages migrations automatically.
