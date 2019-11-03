import React, { useEffect, useState, useContext } from "react";
import {gameConnectionSingleton} from "../connection/index";
import { GameClient } from "inter-planet-game";
import { ModeContext } from "../state/mode";
import GameForm from "./GameForm";

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

    return <>
        <GameForm />
        <div className="game-list">
            {games.map((game) => (<div className="game-card" key={game.game_id} onClick={() => {
                handleGameSelection(game);
            }}>
                {game.game_id}
            </div>))}
        </div>
    </>;
}

export default GameList;