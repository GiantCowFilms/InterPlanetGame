import React from 'react';

interface Props {
    game: any
}

function GameWindow (props: Props) {
    return (
        <div>
            <canvas>
            </canvas>
        </div>
    );
}

export default GameWindow;