import React, { useEffect, useState } from "react";

interface Props {

}

function GameList() {
    const [games, setGames] = useState([]);
    useEffect(() => {
        const socket = new WebSocket("ws://localhost:1234");
        socket.addEventListener("open", function (event) {
        });
        socket.addEventListener("message", function (event) {
            try {
                const message = JSON.parse(event.data);
                if (message["NewGame"] !== undefined) {
                    setGames([...games, message["NewGame"]])
                }
                if (message["GameList"] !== undefined) {
                    setGames([...games, ...message["GameList"].games]);
                }
            } catch (e) {
                console.error(e);
            }
            console.log(event.data);
        });
    }, []);

    return <>
        {games.map((game) => (<div key={game.game_id}>
            {game.game_id}
        </div>))}
    </>;
}

export default GameList;