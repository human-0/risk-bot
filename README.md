# Risk Bot
A bot made for the [SYNCS Bot Battle 2024](https://github.com/syncs-usyd/risk-game-engine).

# Project Structure
* `attack_game` contains the majority of the implementation of the attacking logic.
* `mcts` contains the core search component of the attacking logic.
* `json_connection` contains tools for connecting to the SYNCS match simulator.
* `puct_bot` contains an entry point intended to be compiled to WASM to connect to the match simulator.
* `risk_bots` contains various full Risk bots, including an (approximate) reimplementation of the SYNCS examples, some early attempts, and the main bot.
* `risk_engine` contains a reimplementation of the SYNCS game engine.
* `risk_helper` contains a reimplementation of the SYNCS helper library.
* `risk_shared` contains a reimplementation of the SYNCS shared library.
* `sprt` contains tools for testing
* `spsa` contains tools for tuning
* `stub.py` contains a stub for loading `puct_bot` for the SYNCS match simulator
* `build.sh` attempts to build the `puct_bot` and integrate it with `stub.py`.


# Building
A Rust compiler is required (Nightly is recommended). To use the provided build script, the `wasm-wasip1` target should be installed. The full project, excluding the Python stub can be build using:
```
cargo build --release
```
The stub can be integrated with the main bot using:
```
./build.sh
```
Another WASM bot can be integrated with the stub using
```
echo "WASM = \"$(base64 --wrap=0 <wasm_file>)\"\n$(cat stub.py)" > bot.py
```

# Limitations
There is a very long list of things the bot is unable to do that it probably should be capable of. These include:
* Splitting troops during attacks
* Targeting or avoiding the top player
* Preventing other players from claiming entire continents during the initial phase
* Intelligently choosing when to redeem cards
