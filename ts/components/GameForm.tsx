import React, { useEffect, useState, useContext, useRef } from "react";
import {gameConnectionSingleton} from "../connection/index";
import { ModeContext } from "../state/mode";

interface Props {

}

function GameForm() {
    useEffect(() => {
        gameConnectionSingleton.onEvent("MapList",() => {
            setMapList(gameConnectionSingleton.client.get_maps());
        });
    },[]);
    const [mapList,setMapList] = useState(gameConnectionSingleton.client.get_maps());
    const [minPlayers,setMinPlayers] = useState(2);
    const [mapId,setMapId] = useState("");
    const previewCanvas = useRef(null);
    useEffect(() => {
        const canvas = previewCanvas.current;
        try {
            console.log(mapId);
            gameConnectionSingleton.client.preview_game(canvas,mapId);
        } catch (e) {
            console.error(e);
            const ctx = canvas.getContext("2d");
            ctx.fillStyle = "#ffffff";
            ctx.fillText(10,10,"Could not render a preview.");
        }
    },[
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
        <div className="game-form-fields card-inside">
            <h3>Create a Game</h3>
            <label>Minimum Players</label>
            <input type="text" pattern="[0-9]*" onChange={e => setMinPlayers(parseInt(e.target.value))} value={minPlayers} />
            <label>Map</label>
            <select onChange={e => setMapId(e.target.value)} value={mapId}>
                <option selected={true} disabled={true} hidden value="">Select Map...</option>
                {mapList.map((map_id: string) => {
                    return <option key={map_id} value={map_id}>{map_id}</option>;
                })}
            </select>
            <div className="button" onClick={createGame}>New Game!</div>
        </div>
    </div>;
}

export default GameForm;