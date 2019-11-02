import React, { useContext, useState } from 'react';
import GameWindow from './components/GameWindow';
import GameList from './components/GameList';
import { mode, ModeContext } from './state/mode';
import PlayerName from './components/PlayerName';
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
    let [mode,setMode] = useState<mode>({
        type: "browse"
    });
    let [playerName,setPlayerName] = useState(undefined);
    return <>
        <div className="title">Inter-Planet Game</div>
        <ModeContext.Provider value={{
            mode,
            setMode
        }} >
            {
                mode.type === "browse" ?
                    <GameList />
                : mode.type === "game" ?
                    playerName !== undefined ?
                        <GameWindow game={mode.game} />
                    : <PlayerName onSubmit={setPlayerName} />
                :
                undefined
            }
        </ModeContext.Provider>
    </>;
}

export default Root;