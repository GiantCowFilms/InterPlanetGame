import { make, GameClient } from "ipg_client";
type GameConnectionStatus = "pending"|"error"|"open"|"init";
type GameConnectionEvent = "ConnectionStatusChange"|string;

type GameConnection = {
    client: GameClient;
    socket: WebSocket;
    status: GameConnectionStatus,
    eventHandlers: Map<string,(() => void)[]>;
    onEvent: (event: GameConnectionEvent, callback: () =>void) => () => void;
    triggerEvent: (event: GameConnectionEvent) => void;
    setStatus: (status: GameConnectionStatus) => void;
};

function configureSocket(socket: WebSocket, client: GameClient, connection: GameConnection, delay = 500) {
    const delayRef = { ref: delay };
    // Send a "status" change if the connection hasn't been opened after 400 ms 
    // This will display a connecting message
    const connectingNotice = setTimeout(() => {
        if (connection.status !== "error") {
            connection.setStatus("pending");
        }
    },1000);
    socket.addEventListener("open",() => {
        connection.setStatus("open");
        clearTimeout(connectingNotice);
        // Send a message to prevent cloudflare from timing out hte connection.
        // Cloudflare times out after 100 seconds, so we are sending a keep alive every 60.
        const interval = setInterval(() => {
            socket.send(JSON.stringify("Ping"));
        }, 1000 * 60 /* one minute */);
        socket.addEventListener("close",() => clearInterval(interval));
        delayRef.ref = 1000;
    });
    socket.addEventListener("message", function (event) {
        const type = client.handle_message(event.data);
        connection.triggerEvent(type);
    });
    socket.addEventListener("close",(error) => {
        connection.setStatus("error");
        clearTimeout(connectingNotice);
        // If the connection fails, retry
        setTimeout(() => {
            const newSocket = new WebSocket(socket.url);
            client.set_socket(newSocket);
            connection.socket = newSocket;
            configureSocket(newSocket,client,connection,Math.max(delayRef.ref * 2,8000));
        },delayRef.ref);
    });
}

export default function createConnection (url: string): GameConnection {
    const socket: WebSocket = new WebSocket(url);
    const client: GameClient = make(socket);
    const connection: GameConnection = {
        client,
        socket,
        eventHandlers: new Map(),
        onEvent: function (this: GameConnection, event, callback) {
            if (!this.eventHandlers.has(event)) {
                this.eventHandlers.set(event,[]);
            }
            this.eventHandlers.get(event).push(callback);
            return () => {
                const events = this.eventHandlers.get(event);
                events.splice(events.indexOf(callback),1);
            }
        },
        triggerEvent: function (this: GameConnection, event) {
            const handlers = this.eventHandlers.get(event);
            if (handlers) {
                for (const eventHanlder of handlers) {
                    eventHanlder();
                }
            }
        },
        setStatus: function (this: GameConnection, status) {
            this.status = status;
            this.triggerEvent("ConnectionStatusChange");
        },
        status: "init"
    };
    connection.onEvent.bind(connection);
    connection.triggerEvent.bind(connection);
    configureSocket(socket,client,connection);
    return connection;
}

/**
 * Upgrades ws url to wss, if the page is currently https, since ws is not permitted on an https page. 
 * @param url 
 */
function matchWebsocketTransportSecurity(url: string) {
    const parsedUrl = new URL(url);
    parsedUrl.protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
    return parsedUrl.toString();
}

export const gameConnectionSingleton: GameConnection = createConnection(matchWebsocketTransportSecurity(process.env.SERVER_URL));