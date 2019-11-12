import React, { useEffect, useState, useContext } from "react";
import {gameConnectionSingleton} from "../connection/index";
import GameForm from "./GameForm";
import GameItem from "./GameItem";

interface Props {

}

function GameList() {
    const [games, setGames] = useState([]);
    useEffect(() => {
        const updateGames = () => {
            setGames(gameConnectionSingleton.client.game_list());
        };
        gameConnectionSingleton.onEvent("GameList", updateGames);
        gameConnectionSingleton.onEvent("NewGame", updateGames);
    }, []);
    return <div className="inner-content start-page">
        <GameForm />
        <div className="game-list">
            {games.map((game) => (<GameItem key={game.game_id} game={game} />))}
        </div>
    </div>;
}

export default GameList;