import { make, GameClient } from "inter-planet-game";

type GameConnection = {
    client: GameClient;
    socket: WebSocket;
    eventHandlers: Map<string,(() => void)[]>;
    onEvent: (event: string,callback: () =>void) => void;
};

export default function createConnection (url: string = "ws://localhost:1234"): GameConnection {
    const client: GameClient = make();
    const socket: WebSocket = new WebSocket(url);
    const connection: GameConnection = {
        client,
        socket,
        eventHandlers: new Map(),
        onEvent: function (this: GameConnection, event, callback) {
            if (!this.eventHandlers.has(event)) {
                this.eventHandlers.set(event,[]);
            }
            this.eventHandlers.get(event).push(callback);
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

export const singleton: GameConnection = createConnection();