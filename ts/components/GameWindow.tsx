import React, { useEffect, useRef, useState } from 'react';
import {gameConnectionSingleton} from "../connection/index";

interface Props {
    game: any
}

function GameWindow (props: Props) {
    const canvas = useRef(null);
    const players = useState([]);
    useEffect(() => {
        gameConnectionSingleton.client.enter_game(props.game);
        gameConnectionSingleton.client.set_render_target(canvas.current);
        let gameStarted = false;
        // game is implictly started when the first GameState is sent
        gameConnectionSingleton.onEvent("Game", () => {
            gameStarted = true;
            gameConnectionSingleton.client.render_game_frame(5000000);
        });
    },[canvas,props.game]);
    return (
        <div>
            <canvas id="game-canvas" ref={canvas} >
            </canvas>
        </div>
    );
}

export default GameWindow;