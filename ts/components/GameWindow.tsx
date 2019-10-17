import React, { useEffect, useRef, useState, MouseEvent } from 'react';
import {gameConnectionSingleton} from "../connection/index";

interface Props {
    game: any
}

function GameWindow (props: Props) {
    const canvasTop = useRef(null);
    const canvasBottom = useRef(null);
    const players = useState([]);
    const [gameStarted,setGameStarted] = useState(false);
    const startGame = () => {
        gameConnectionSingleton.client.start_game();
    }
    const MouseEvent = (e: MouseEvent) => {
        // This is awful:
        const rect = (Array.from(canvasTop.current.parentElement.children)
            .find((child: any) => child.id === "game-canvas-top") as any)
            .getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        gameConnectionSingleton.client.mouse_event(x,y);
    }
    useEffect(() => {
        gameConnectionSingleton.client.enter_game(props.game);
        gameConnectionSingleton.client.set_render_target(canvasTop.current,canvasBottom.current);
        // game is implictly started when the first GameState is sent
        gameConnectionSingleton.onEvent("Game", () => {
            setGameStarted(true);
            gameConnectionSingleton.client.render_game_frame(5000000);
        });
    },[canvasTop,canvasBottom,props.game]);
    return (
        <div style={{
            "position": "relative"
        }}>
            {gameStarted ? undefined : <div onClick={startGame}>Start Game!</div>}
            <canvas id="game-canvas-top" ref={canvasTop} style={{
                "position": "absolute"
            }} onMouseDown={MouseEvent} onMouseUp={MouseEvent}>
            </canvas>
            <canvas id="game-canvas-bottom" ref={canvasBottom} >
            </canvas>
        </div>
    );
}

export default GameWindow;