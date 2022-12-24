import React, { useState } from 'react';
import {gameConnectionSingleton} from "../connection/index";

interface Props {
    onSubmit: (name: string) => void
}

function PlayerName (props: Props) {
    const [playerName,setPlayerName] = useState("");
    const setName = () => {
        props.onSubmit(playerName);
    };
    return (
        <div className="center-content">
            <div className="card card-inside player-name">
                <h3>Choose an Alias</h3>
                <input onChange={(e) => setPlayerName(e.target.value)} value={playerName} />
                <div onClick={setName} className="button">
                    Save
                </div>
            </div>
        </div>
    );
}

export default PlayerName;