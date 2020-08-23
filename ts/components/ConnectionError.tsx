import React, { useEffect, useState } from "react";
import { gameConnectionSingleton } from "../connection";

const ConnectionError: React.FC = () => {
    const [error, setError] = useState(false);
    useEffect(() => {
        const remove1 = gameConnectionSingleton.onEvent("ConnectionClose", () => {
            setError(true);
        });
        const remove2 = gameConnectionSingleton.onEvent("ConnectionOpen", () => {
            setError(false);
        });
        return () => {
            remove1(); 
            remove2();
        };
    }, [setError]);
    if (error) {
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
    return null;
}

export default ConnectionError;