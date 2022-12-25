import React, { useEffect, useState, useContext, useRef } from "react";
import {gameConnectionSingleton} from "../connection/index";
import { game, ModeContext } from "../state/mode";

interface Props {
    game: game
}

function GameItem(props: Props) {
    const [games, setGames] = useState([]);
    useEffect(() => {
        gameConnectionSingleton.onEvent("GameList", () => {
            setGames(gameConnectionSingleton.client.game_list());
        });
    }, []);

    const modeContext = useContext(ModeContext);

    const handleGameSelection = (game: game) => {
        modeContext.setMode({
            type: "game",
            game
        })
    };
    const preview = useRef(null);
    useEffect(() => {
        if (preview != null) {
            gameConnectionSingleton.client.preview_game(preview.current,props.game.map_id);
        }
    },[props.game])

    return <>
            <div className="game-item card" key={props.game.game_id} onClick={() => {
                handleGameSelection(props.game);
            }}>
                <canvas className="card-inset" ref={preview}>

                </canvas>
                <div className="card-inside">
                    {props.game.game_id}
                </div>
            </div>
    </>;
}

export default GameItem;