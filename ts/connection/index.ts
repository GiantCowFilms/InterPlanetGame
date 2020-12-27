import { make, GameClient } from "ipg_client";

type GameConnection = {
    client: GameClient;
    socket: WebSocket;
    eventHandlers: Map<string,(() => void)[]>;
    onEvent: (event: string,callback: () =>void) => () => void;
    triggerEvent: (event: string) => void;
};

function configureSocket(socket: WebSocket, client: GameClient, connection: GameConnection, delay = 1000) {
    const delayRef = { ref: delay };
    socket.addEventListener("open",() => {
        connection.triggerEvent("ConnectionOpen");
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
        connection.triggerEvent("ConnectionClose");
        // If the connection fails, retry
        setTimeout(() => {
            const newSocket = new WebSocket(socket.url);
            client.set_socket(newSocket);
            connection.socket = newSocket;
            configureSocket(newSocket,client,connection,delayRef.ref * 2);
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
        }
    };
    connection.onEvent.bind(connection);
    connection.triggerEvent.bind(connection);
    configureSocket(socket,client,connection);
    return connection;
}

export const gameConnectionSingleton: GameConnection = createConnection(process.env.SERVER_URL);