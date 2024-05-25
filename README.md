# algo-trading

## Todo

- Basic mean-reversion strategy
- Backtesting
  - Simulate a market data feed and see what happens!
- OrderService
  - Use the Tradier API for placing orders
  - Track positions independently and reconcile with Tradier on startup
  - Compute P&L on every sell and store in MongoDB

## Dependendcy Injection

- Will use dynamic dispatch against traits
- Implementations chosen based on both config and unit tests
- Service construction will be done in main.rs:
  - Config will be read from a file
  - Services will be constructed based on the config, hierarchal including dependencies
    - Service parameters (to other services) must be wrapped in Arc
    - And they must be thread-safe - or use channels
    - Each service will get its config as a map
  - Services will be started as needed (MarketData init, TradingService loop...)

## Basic Architecture

- Communication between services will be done via channels for decoupling and maximum efficiency (make use of many cores)
- One MarketData service - Tradier's streaming data sends everything, it seems
- N TradingServices - one per strategy - again to maximize efficiency
  - Each TradingService will have a TradingStrategy, where the actual logic is
  - Driven by config - the strategy type and its parameters (symbols, etc)
- One OrderService - will be a simple wrapper around Tradier's API
  - Will track positions (MongoDB) independently also

### Communication

- Direct fn calls will require locking
- Decoupling with channels means spawning threads but no locking, also asynch for better performance

### TradingService

- Subscribes to all passed-in MarketData sources
  - On creation? If so, it has no public methods...
- Exists to wrap a Strategy, which is what makes trading decisions
- Subscribe to MarketDataService, then loop on rx.recv()
  - Processes the data
  - Makes trading decisions via its attached TradingStrategy
  - Sends orders to OrderService

# Family

## Members

### My daughter Mary likes to lay in her bed and play with her iphone and then tell tall tales about it.

She also loves her chicken, named "Taki." He is a sturdy, mannish rooster who is always on the lookout for a fight. He is a good protector of the hens, but he is also a bit of a bully. He is always trying to get the best food and the best spot in the coop. He is a bit of a show-off, too. He struts around the yard, puffing out his chest and crowing loudly. He is a handsome bird, with shiny black feathers and bright red comb. He is a good rooster, but he can be a bit of a handful.
