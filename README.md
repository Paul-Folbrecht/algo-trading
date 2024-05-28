# algo-trading

## Onion Architecture

### https://dev.to/jnavez/make-your-microservices-tastier-by-cooking-them-with-a-sweet-onion-34n2

- The center of the graph is the domain model: our business objects and the relations between those objects.

- Around the domain model, we find our domain services. This is the business logic, our use-cases. The domain services strongly rely on the domain objects.

  - This is the Service pattern, essentially

- The controllers (and the scheduled jobs) are the entry modules of the architecture. Thus they need to call the domain services to respond to the frontend or other clients.

  - No UI at the moment

- The persistence module and the clients module are part of the infrastructure. Here, “Infrastructure” is a conceptual term used to group all the modules related to persistence, external communications...

  - Why is persistence "special"? Why can't persistence be just another service? Or the individual services handle their own persistence? Why would an external persistence service go through the services layer (as depicted in the diagram) rather than right to domain?

### https://github.com/pocket7878/rust-onion-example

- Uses workspaces!
- domain has pub structs and their impls - as expected
- infra is the services layer - TaskRdbRepository - a trait and its impl as expected

### JdG

- Services should be an interface + ADTs for inputs and outputs.
- Implement all services with [trait objects] in terms of other services.
- "This can be thought of as a translation from a higher-level to lower-level language." Implement inner layer in terms of the next layer out. Don't "hop layers."
- Every service should be a final case class with interface parameters.
- "At the edge of the onion, your constructor list is empty" - no dependent services there. File system, socket, etc.
- "The outermost layer is the only one that knows about the outside world."
- Service interfaces should be in their own compilation unit (src tree) - is this necessary in Rust?

### Refactoring

- Create domain, move all public structs and impls there
- Create services, move all services there
- Create core/util - serde goes there for now
- Create workspace Cargo.toml files

## Todo

- Make messaging a service?

- Basic mean-reversion strategies:
  - 30d vs 90d moving averages
  - single window - check for 2 std deviation from mean (Bollinger Bands)
- OrderService - Initial
  - Track positions independently and reconcile with Tradier on startup
  - Compute P&L on every sell and store in MongoDB
- OrderService
  - Use the Tradier API for placing orders real
- Backtesting
  - Simulate a market data feed and see what happens!

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
