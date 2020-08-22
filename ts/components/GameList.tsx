import React, { useEffect, useState, useContext } from "react";
import {gameConnectionSingleton} from "../connection/index";
import GameForm from "./GameForm";
import GameItem from "./GameItem";
import { useGameList } from "../connection/hooks";

interface Props {

}

function GameList() {
    const [games, setGames] = useGameList();
    return <div className="inner-content start-page">
        <GameForm />
        <div className="game-list">
            {games.map((game) => (<GameItem key={game.game_id} game={game} />))}
        </div>
    </div>;
}

export default GameList;