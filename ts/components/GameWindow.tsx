import React, { useEffect, useRef } from 'react';
import {gameConnectionSingleton} from "../connection/index";

interface Props {
    game: any
}

function GameWindow (props: Props) {
    const canvas = useRef(null);
    useEffect(() => {
        gameConnectionSingleton.client.enter_game(props.game);
        gameConnectionSingleton.client.set_render_target(canvas);
    },[canvas,props.game]);
    return (
        <div>
            <canvas id="game-canvas" ref={canvas}>
            </canvas>
        </div>
    );
}

export default GameWindow;