import { make, GameClient } from "ipg_client";

type GameConnection = {
    client: GameClient;
    socket: WebSocket;
    eventHandlers: Map<string,(() => void)[]>;
    onEvent: (event: string,callback: () =>void) => () => void;
};

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
        }
    };
    connection.onEvent.bind(connection);
    socket.addEventListener("message", function (event) {
        const type = client.handle_message(event.data);
        let handlers = connection.eventHandlers.get(type);
        if (handlers) {
            for (let eventHanlder of handlers) {
                eventHanlder();
            }
        }
    });
    return connection;
}

export const gameConnectionSingleton: GameConnection = createConnection(process.env.SERVER_URL);