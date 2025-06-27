# Click Trader Server

`click-trader-server` is the backend for a real-time incremental game (clicker game) with a dynamic stock market, built using Rust and SpacetimeDB. This server handles player data, manages stock market simulations, processes transactions, and applies player upgrades.

## Features

*   **Player Management:**
    *   Handles player connections and disconnections.
    *   Manages player money, passive income, click power, and stock holdings.
    *   Allows players to set their usernames.
    *   Processes player clicks to generate income.
*   **Dynamic Stock Market:**
    *   Simulates a real-time stock market with multiple companies.
    *   Stock prices fluctuate based on recent buy and sell activities.
    *   Configurable market sensitivity and slippage.
*   **Transactions:**
    *   Supports buying and selling of stocks.
    *   Applies buy and sell fees to transactions.
    *   Manages pending, confirmed, and rejected transactions.
*   **Upgrades System:**
    *   Allows players to purchase upgrades to boost passive income, click power, and reduce click timer.
    *   Pre-defined upgrades with varying costs and effects.

## Technologies Used

*   **Rust:** A systems programming language focused on safety, performance, and concurrency.
*   **SpacetimeDB:** A real-time, collaborative database that enables building multiplayer applications with ease.

## Project Structure

The core logic of the server is organized into several modules within the `src/` directory:

*   `constants.rs`: Defines various constant values used throughout the application, such as starting player stats, update intervals, and scaling factors.
*   `initializer.rs`: Contains functions responsible for initializing the game state, including market configuration, pre-defined upgrades, and initial stocks.
*   `lib.rs`: The main library file that defines the SpacetimeDB reducers for `init`, `client_connected`, and `client_disconnected` events, setting up initial schedules for player and stock market updates.
*   `player_module.rs`: Manages player-related data and logic, including player state, updating player income, and handling username changes and click-based money generation.
*   `stock_module.rs`: Implements the stock market mechanics, including stock creation, price updates based on supply and demand, and market configuration.
*   `transaction_module.rs`: Handles the creation and processing of stock buy and sell transactions, including fee calculation and updating player and stock data.
*   `upgrades_module.rs`: Manages the upgrade system, allowing players to purchase upgrades and applying their effects to player stats.

## Setup and Installation

To set up and run the `click-trader-server`, you will need to have Rust and Cargo installed, as well as the SpacetimeDB CLI.

1.  **Clone the repository:**
    ```bash
    git clone git@github.com:Telikz/click-trader-server.git
    cd click-trader-server
    ```
    
2. **Deploy to SpacetimeDB:**
    You will need to use the SpacetimeDB CLI to deploy this module. Ensure you have a SpacetimeDB instance running or configured.
    ```bash
    spacetime publish -s server-name module-name
    ```