import React, { useEffect, useState } from "react";
import { gameConnectionSingleton } from "../connection";

const ConnectionStatus: React.FC = ({ children }) => {
    const [status, setStatus] = useState("notstarted");
    useEffect(() => {
        const remove = gameConnectionSingleton.onEvent("ConnectionStatusChange", () => {
            setStatus(gameConnectionSingleton.status);
        });
        return () => {
            remove();
        };
    }, [setStatus]);
    if (status == "error") {
        return <div className="card card-inside error inner-content">
            <h3>Connection Error</h3>
            <p>
                Could not connect to the server. If you are on a corporate VPN, it is probably blocking the websocket connection.
            </p>
            <p>
                Trying to reconnect...
            </p>
        </div>;
    }
    if (status == "pending") {
        return <div className="card card-inside inner-content">
            <h3>Connecting...</h3>
            <p>
                Attempting to reach the game server. If this takes a long time, check your interent and try using a different browser. If you are on a corporate VPN, it may be blocking the websocket connection.
            </p>
        </div>;
    }
    if (status == "open") {
        return <>{children}</>;
    }
    return null;
}

export default ConnectionStatus;