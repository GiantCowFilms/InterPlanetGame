import React, { useState } from 'react';
import {gameConnectionSingleton} from "../connection/index";

interface Props {
    onSubmit: (name: string) => void
}

function PlayerName (props: Props) {
    const [playerName,setPlayerName] = useState("");
    const setName = () => {
        gameConnectionSingleton.client.set_name(playerName);
        props.onSubmit(playerName);
    };
    return (
        <div className="card card-inside">
            <h3>Choose an Alias</h3>
            <input onChange={(e) => setPlayerName(e.target.value)} value={playerName} />
            <div onClick={setName} className="button">
                Save
            </div>
        </div>
    );
}

export default PlayerName;