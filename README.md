# algo-trading

Algorithmic trading strategies and backtesting in Rust.

To obtain an access token, create an account at Tradier, then go to [this page](https://documentation.tradier.com/brokerage-api/oauth/access-token).

## Configuration

At present, only a very simple Bolinger Bands strategy is implemented.

Modify either `config\default.toml` or `config\local.toml` to configure symbols. Example:

```
[[strategies]]
name = "mean-reversion"
symbols = ["SPY", "AAPL"]
```

## Building

`cargo build`

## Running

`cargo run <access-token>`
