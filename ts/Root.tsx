import React from 'react';
import GameWindow from './components/GameWindow';
import GameList from './components/GameList';
type game_state = {

}
type game_metadata = {

}
type state = {
    currentGame?: game_metadata,
    gameState?: game_state,
    playerName?: string,
    game_list: game_metadata[]
}

function Root() {
    return <>
        <div>Inter-Planet Game</div>
        <GameWindow />
        <GameList />
    </>;
}

export default Root;