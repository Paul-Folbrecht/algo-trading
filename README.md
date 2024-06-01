# algo-trading

## Basic Architecture

- Service model so we can use DI for testing - otherwise bare fns would have been fine
- Communication between services will be done via channels for decoupling and maximum efficiency (make use of many cores)
- One MarketData service - Tradier's streaming data sends everything, it seems
- N TradingServices - one per strategy - again to maximize efficiency
  - Each TradingService will have a TradingStrategy, where the actual logic is
  - Driven by config - the strategy type and its parameters (symbols, etc)
- One OrderService - will be a simple wrapper around Tradier's API
- Will track positions (MongoDB) independently also
