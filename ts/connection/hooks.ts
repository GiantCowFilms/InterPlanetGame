import { useEffect, useState } from "react";
import { gameConnectionSingleton } from ".";

export const useGameList = () => {
    const state = useState([]);
    const [games,setGames] = state;
    useEffect(() => {
        const updateGames = () => {
            setGames(gameConnectionSingleton.client.game_list());
        };
        const remove1 = gameConnectionSingleton.onEvent("GameList", updateGames);
        const remove2 = gameConnectionSingleton.onEvent("NewGame", updateGames);
        return () => {
            remove1();
            remove2();
        };
    },[setGames]);
    return state;
}