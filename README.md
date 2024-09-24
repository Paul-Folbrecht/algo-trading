# algo-trading

Algorithmic trading strategies and backtesting in Rust.

To obtain an access token, create an account at Tradier, then go to [this page](https://documentation.tradier.com/brokerage-api/oauth/access-token).

DISCLAIMER: This is a personal project and is not intended for production use. Use at your own risk. The author is not responsible for any financial losses incurred as a result of using this software.

## Configuration

At present, only a very simple Bolinger Bands strategy is implemented.

Either modify `config\default.toml` or create `config\local.toml` to specify your account and trading information.

Example:

```
sandbox = true
hist_data_range = 20
backtest_range = 730

[[strategies]]
name = "mean-reversion"
symbols = ["AAPL", "AMZN"]
capital = [100000, 10000]
```

`sandbox_token` must be set, but a valid value is optional and only required if you are using the sandbox environment.

## Building

`cargo build` is all you need.

## Environment

The following environment variables must be set:

ACCESS_TOKEN
SANDBOX_TOKEN
ACCOUNT_ID
MONGO_URL

(Environment rather than config for secrets per the [12-factor app](https://12factor.net/config) methodology - although I use config for non-secret settings.)

## Running Locally

`./scripts/run.sh`

or

`cargo run --bin server`

## MongoDB

- Install locally: `brew tap mongodb/brew && brew install mongodb-community`
- Start: `brew services start mongodb-community`
- Shell: `mongo`

In the shell, execute `use algo-trading` to use the database. Queries on collections positions, orders, and pnl can then be made.

Alternatively, you can use a MongoDB Atlas instance - just set `mongo_url` to an appropriate connection string.

Note: A MongoDB Atlas connection string (MONGO_URL environment variable) should be of the form

```
mongodb://[username:password@]host1[:port1][,...hostN[:portN]][/[defaultauthdb][?options]]
```

Example:

```
export MONGO_URL="mongodb+srv://{username}:{password}@cluster0.31ie7.mongodb.net/?retryWrites=true&w=majority&appName=Cluster0"
```

## Testing

`cargo test` requires the environment variables `TRADIER_ACCESS_TOKEN`, `TRADIER_SANDBOX_TOKEN`, and `TRADIER_ACCOUNT_ID` to be set.

## Backtesting

`./scripts/backtest.sh`

or

`cargo run --bin backtest`

Output will include generated realized P&L and open positions.

## Docker

To build the image:

`docker build -t algo-trading .`

To tag for GCP Registry:

```
docker tag \
    algo-trading \
    us-central1-docker.pkg.dev/{project-id}/{repo-name}/algo-trading
```

And to push:

`
docker image push us-central1-docker.pkg.dev/{project-id}/{repo-name}/algo-trading:latest
`
