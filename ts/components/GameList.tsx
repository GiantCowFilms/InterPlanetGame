import React, { useEffect, useState, useContext } from "react";
import {gameConnectionSingleton} from "../connection/index";
import { GameClient } from "inter-planet-game";
import { ModeContext } from "../state/mode";

interface Props {

}

function GameList() {
    const [games, setGames] = useState([]);
    useEffect(() => {
        gameConnectionSingleton.onEvent("GameList", () => {
            setGames(gameConnectionSingleton.client.game_list());
        });
    }, []);

    const modeContext = useContext(ModeContext);

    const handleGameSelection = (game: any) => {
        modeContext.setMode({
            type: "game",
            game
        })
    };

    const createGame = () => {
        gameConnectionSingleton.socket.send(JSON.stringify({
            "CreateGame": {
                map_id: "Example Map",
                config: {
                    min_players: 2
                }
            }
        }));
    }

    return <>
        {games.map((game) => (<div key={game.game_id} onClick={() => {
            handleGameSelection(game);
        }}>
            {game.game_id}
        </div>))}
        <div onClick={createGame}>New Game!</div>
    </>;
}

export default GameList;