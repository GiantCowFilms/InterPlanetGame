# Inter Planet Game (WIP)

http://interplanetgame.giantcowfilms.com

A fun multi-player game of galaxy conquest!

## How to Play

Create a game using the controls at the top of the start page, or join an existing game listed below. Once a suffcient number of players have joined a game, the option will appear to start the game. Start it.

The goal is to take possesion of all the planets in the game. Planets can be invaded by dragging from a controlled planet to an uncontrolled planet. Half the troops on the source planet will leave in the attack. Each troop will reduce the hostile troop count by one. If a troop lands while zero hostile troops are on a planet, it will claim that planet. Troops can also be redistrbuted between controlled planets by dragging from a controlled planet to another controlled planets. Controlled planets will match the color if the icon next to a player's name. Each planet generates troops, the bigger the planet, the faster the troop generation. Neutral planets (grey) do not generate neutral troops. They troop count will remain static until they have been taken over by a player.

## About the Code

The core game logic (./core) is implemented in Rust, as is the server (./server) and the parts of the client (./client) focused on rendering the game. The UI is TypeScript/React (./ts). The Rust portions that run in the browser are compiled into WebAssembly.

### Local dev setup
You need to install:
 * Rust - https://www.rust-lang.org/tools/install
 * wasm-pack - https://rustwasm.github.io/wasm-pack/
 * Node & npm

run 



