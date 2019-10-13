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
        <div>
            <input onChange={(e) => setPlayerName(e.target.value)} value={playerName} />
            <div onClick={setName}>
                Save
            </div>
        </div>
    );
}

export default PlayerName;