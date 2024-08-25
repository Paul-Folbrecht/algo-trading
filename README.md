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
access_token = "XXXX"
sandbox_token = "XXXX"
account_id = "XXXX"
hist_data_range = 20
backtest_range = 730
mongo_url = "mongodb://localhost:27017/"

[[strategies]]
name = "mean-reversion"
symbols = ["AAPL", "AMZN"]
capital = [100000, 10000]
```

`sandbox_token` must be set, but a valid value is optional and only required if you are using the sandbox environment.

## Building

`cargo build`

## Running Locally

`./run.sh`

or

`cargo run --bin server`

## MongoDB

- Install locally: `brew tap mongodb/brew && brew install mongodb-community`
- Start: `brew services start mongodb-community`
- Shell: `mongo`

In the shell, execute `use algo-trading` to use the database. Queries on collections positions, orders, and pnl can then be made.

Alternatively, you can use a MongoDB Atlas instance - just set `mongo_url` to an appropriate connection string.

## Testing

`cargo test` requires the environment variables `TRADIER_ACCESS_TOKEN`, `TRADIER_SANDBOX_TOKEN`, and `TRADIER_ACCOUNT_ID` to be set.

## Backtesting

`cargo run --bin backtest`

Output will include generated realized P&L and open positions.
