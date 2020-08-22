import React, { useContext, useState, useEffect } from 'react';
import GameWindow from './components/GameWindow';
import GameList from './components/GameList';
import { mode, ModeContext } from './state/mode';
import PlayerName from './components/PlayerName';
import { useGameList } from './connection/hooks';
import { gameUrl } from './gameInfo';
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
    let [mode, setModeInternal] = useState<mode>({
        type: "browse"
    });
    const setMode = (mode: mode) => {
        if (mode.type === "game") {
            window.location.hash = gameUrl(mode.game).hash;
        } else if (mode.type === "browse") {
            window.location.hash = "";
        }
        setModeInternal(mode);
    };
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
                if (game && (mode.type !== "game" || mode.game !== game)) {
                    setMode({
                        type: "game",
                        game
                    });
                }
            }
        };
        onHashChange();
        document.addEventListener("hashchange", onHashChange);
        return () => {
            document.removeEventListener("hashchange", onHashChange);
        };
    },[games]);
    let [playerName, setPlayerName] = useState(undefined);
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