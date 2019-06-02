import React from 'react';

interface Props {

}

class GameWindow extends React.Component<Props, {}> {

    componentDidMount() {
        const socket = new WebSocket('ws://localhost:1234');
        socket.addEventListener('open', function (event) {
            socket.send('Foo Bar');
            socket.send(JSON.stringify({
                "ExitGame": null
            }));
            socket.send(JSON.stringify({
                "CreateGame": {
                    map_id: "Example Map"
                }
            }));
        });
        socket.addEventListener('message', function (event) {
            console.log(event.data);
        });
    }

    render() {
        return (
            <div>
                <canvas>
                </canvas>
            </div>
        );
    }
}

export default GameWindow;