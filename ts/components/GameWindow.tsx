import React, { useEffect, useRef, useState, MouseEvent } from 'react';
import { gameConnectionSingleton } from "../connection/index";
import { playerColors, gameUrl } from '../gameInfo';

interface Props {
    game: any
}

function GameWindow(props: Props) {
    const canvasTop = useRef(null);
    const canvasBottom = useRef(null);
    const [players, setPlayersInternal] = useState([]);
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
        gameConnectionSingleton.client.enter_game(props.game, canvasTop.current, canvasBottom.current);
        // game is implictly started when the first GameState is sent
        let renderStarted = false;
        const unHookGameEvent = gameConnectionSingleton.onEvent("Game", () => {
            if (!renderStarted) {
                const gameTimeFrames = gameConnectionSingleton.client.get_time(); // warning: nullable
                const startTimeMilliseconds = Date.now() - (gameTimeFrames * 17);
                console.log(`Started at: ` + startTimeMilliseconds);
                const render = () => {
                    const time = ~~((Date.now() - startTimeMilliseconds) / 17);
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
        const unHookGamePlayersEvent = gameConnectionSingleton.onEvent("GamePlayers", setPlayers);
        return () => {
            unHookGameEvent();
            unHookGamePlayersEvent();
        };
    }, [canvasTop, canvasBottom, props.game]);
    const canStart = props.game.config.min_players <= players.length;
    return (
        <>
            <div style={{
                "position": "relative"
            }}>
                <div className="game-waiting">
                    {gameStarted ? undefined : <>
                        {!canStart ?
                            <div>
                                <h2>Waiting for more players to join...</h2>
                                <div>({players.length}/{props.game.config.min_players})
                                players have joined.</div>
                                <div>Invite your friends! <input readOnly value={gameUrl(props.game).toString()} /></div>
                            </div>
                             :
                             <div>
                                 <h2>Ready to begin...</h2>
                                 <div>{players.length} players have joined.</div>
                                 <div onClick={startGame} className={["button", !canStart ? "disabled" : ""].join(" ")}>Start Game!</div>
                             </div>
                        }
                    </>}
                </div>
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
                    return <div>
                        <div
                            className="player-color"
                            style={{ backgroundColor: playerColors[player.possession + 1] }} />
                        {player.name}</div>;
                })}
            </div>
        </>
    );
}

export default GameWindow;