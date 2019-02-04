import React from 'react';

interface Props {

}

class GameWindow extends React.Component<Props, {}> {

    componentDidMount() {
        const socket = new WebSocket('ws://localhost:1234');
        socket.addEventListener('open', function (event) {
           socket.send('Foo Bar');
        });
        socket.addEventListener('message', function (event) {
            console.log(event.data);
        });
    }

    render() {
        return (
            <div>
                Foo Bar
                <canvas>
                </canvas>
            </div>
        );
    }
}

export default GameWindow;