import React, { useEffect, useState, useContext, useRef } from "react";
import { gameConnectionSingleton } from "../connection/index";
import { ModeContext } from "../state/mode";

interface Props {

}

function GameForm() {
    useEffect(() => {
        const remove = gameConnectionSingleton.onEvent("MapList", () => {
            setMapList(gameConnectionSingleton.client.get_maps());
        });
        return remove;
    }, []);
    const [mapList, setMapList] = useState(gameConnectionSingleton.client.get_maps());
    const [minPlayers, setMinPlayers] = useState(2);
    const [mapId, setMapId] = useState("");
    const previewCanvas = useRef(null);
    useEffect(() => {
        const canvas = previewCanvas.current;
        try {
            console.log(mapId);
            gameConnectionSingleton.client.preview_game(canvas, mapId);
        } catch (e) {
            console.error(e);
            const ctx = canvas.getContext("2d");
            ctx.fillStyle = "#ffffff";
            ctx.fillText(10, 10, "Could not render a preview.");
        }
    }, [
        mapId
    ]);
    const createGame = () => {
        gameConnectionSingleton.socket.send(JSON.stringify({
            "CreateGame": {
                map_id: mapId,
                config: {
                    min_players: minPlayers
                }
            }
        }));
    };
    return <div className="game-form card">
        <canvas ref={previewCanvas} className="card-inset">

        </canvas>
        <div className="game-form-container card-inside">
            <div className="game-form-fields">
                <h3>Create a Game</h3>
                <label>Minimum Players</label>
                <input type="text" pattern="[0-9]*" onChange={e => setMinPlayers(parseInt(e.target.value))} value={minPlayers} />
                <label>Map</label>
                <select onChange={e => setMapId(e.target.value)} value={mapId}>
                    <option disabled={true} hidden value="">Select Map...</option>
                    {mapList.map((map_id: string) => {
                        return <option key={map_id} value={map_id}>{map_id}</option>;
                    })}
                </select>
                <div className="button" onClick={createGame}>New Game!</div>
            </div>
            <div>
                <h3>How to Play</h3>
                <p>
                    To start playing, select an existing game to join below, or create a new one.
                    To create a new game, choose a map and then select <b>New Game!</b>
                    Once enough players have joined the game, select start game to begin playing.
                    Look in the lower right-hand corner to see what color you are. Planets of that
                    color are controlled by you.
                </p>
                <p>
                    You can send ships to other planets by dragging from
                    a planet you control to another planet. This will send half of the ships on the planet
                    to the target planet. If the target planet is controlled by an opponent,
                    those ships will subtract ships from the opposing planet.
                    If you push the enemy troop count below zero, you will take over the planet.
                </p>
                <p>
                    You win the game by taking control of all the planets. Good luck, Admiral!
                </p>
            </div>
        </div>
    </div>;
}

export default GameForm;