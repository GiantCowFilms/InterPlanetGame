import React, { useContext, useState, useEffect } from 'react';
import GameWindow from './components/GameWindow';
import GameList from './components/GameList';
import { mode, ModeContext } from './state/mode';
import PlayerName from './components/PlayerName';
import { useGameList } from './connection/hooks';
import { gameUrl } from './gameInfo';
import ConnectionStatus from './components/ConnectionStatus';
import { useStorageState } from './util/hooks';
import { gameConnectionSingleton } from './connection';
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
    const [mode, setModeInternal] = useState<mode>({
        type: "browse"
    });
    const setMode = ((newMode) => {
        setModeInternal(oldMode => {
            const mode = typeof newMode === 'function' ? newMode(oldMode) : newMode;
            if (mode.type === "game") {
                window.location.hash = gameUrl(mode.game).hash;
            } else if (mode.type === "browse") {
                window.location.hash = "";
            }
            return mode;
        });
    }) as typeof setModeInternal;
    const [games] = useGameList();
    useEffect(() => {
        const onHashChange = () => {
            const hash = window.location.hash;
            const parsed = new Map(hash
                .substr(1)
                .split(";")
                .map(
                    s => s.split("=").slice(0, 2) as [string, string]
                )
            );
            const join = parsed.get("join");
            if (join) {
                const game = games.find(game => game.game_id === join);
                // todo display error if game is not found
                if (game) {
                    setMode(mode => { 
                        if (mode.type !== "game" || mode.game.game_id !== game.game_id) {
                            return {
                                type: "game",
                                game
                            }
                        } else {
                            return mode;
                        }
                    });
                }
            }
        };
        onHashChange();
        document.addEventListener("hashchange", onHashChange);
        return () => {
            document.removeEventListener("hashchange", onHashChange);
        };
    },[games,mode]);
    const [playerName, setPlayerName] = useStorageState("playerName");
    useEffect(() => {
        if (playerName === undefined) return;
        if(gameConnectionSingleton.status !== "open") {
            const remove = gameConnectionSingleton.onEvent("ConnectionStatusChange",() => {
                gameConnectionSingleton.client.set_name(playerName);
            });
            return remove;
        } else {
            gameConnectionSingleton.client.set_name(playerName);
        };
    },[playerName])
    return <>
        <div className="title">Inter-Planet Game</div>
        <ConnectionStatus>
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
        </ConnectionStatus>
    </>;
}

export default Root;