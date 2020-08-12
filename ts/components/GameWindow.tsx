import React, { useEffect, useRef, useState, MouseEvent } from 'react';
import { gameConnectionSingleton } from "../connection/index";

interface Props {
    game: any
}

function GameWindow(props: Props) {
    const canvasTop = useRef(null);
    const canvasBottom = useRef(null);
    const [players,setPlayersInternal] = useState([]);
    const setPlayers = () => setPlayersInternal(gameConnectionSingleton.client.get_player_list());
    const [gameStarted, setGameStarted] = useState(false);
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
        gameConnectionSingleton.client.mouse_event(x, y);
    }
    useEffect(() => {
        gameConnectionSingleton.client.enter_game(props.game);
        gameConnectionSingleton.client.set_render_target(canvasTop.current, canvasBottom.current);
        // game is implictly started when the first GameState is sent
        let renderStarted = false;
        const unHookGameEvent = gameConnectionSingleton.onEvent("Game", () => {
            if(!renderStarted) {
                const gameTimeFrames = gameConnectionSingleton.client.get_time(); // warning: nullable
                const startTimeMilliseconds = Date.now() - (gameTimeFrames * 17);
                console.log(`Started at: ` + startTimeMilliseconds);
                const render = () => {
                    const time = ~~((Date.now() - startTimeMilliseconds)/17);
                    if (time >= 0) {
                        gameConnectionSingleton.client.render_game_frame(time);
                    }
                    window.requestAnimationFrame(render);
                };
                window.requestAnimationFrame(render);
                renderStarted = true;
            }
            setGameStarted(true);
            setPlayers();
        });
        const unHookGamePlayersEvent = gameConnectionSingleton.onEvent("GamePlayers",setPlayers);
        return () => {
            unHookGameEvent();
            unHookGamePlayersEvent();
        };
    }, [canvasTop, canvasBottom, props.game]);
    return (
        <>
            <div style={{
                "position": "relative"
            }}>
                {gameStarted ? undefined : <div onClick={startGame} className="button">Start Game!</div>}
                <canvas id="game-canvas-top" ref={canvasTop} style={{
                    "position": "absolute"
                }} onMouseDown={MouseEvent} onMouseUp={MouseEvent}>
                </canvas>
                <canvas id="game-canvas-bottom" ref={canvasBottom} >
                </canvas>
            </div>
            <div className="card card-inside game-players"> 
                <h4>Players</h4>
                {players.map(player => {
                    return <div>{player.name}</div>;
                })}
            </div>
        </>
    );
}

export default GameWindow;