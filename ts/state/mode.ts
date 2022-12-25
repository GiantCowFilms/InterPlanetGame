import React from "react";

export type game = {
    game_id: string,
    map_id: string,
};

export type mode = {
    type: "game",
    game: game,
} | {
    type: "browse"
};

export const ModeContext = React.createContext<{
    mode: mode,
    setMode: (mode: mode) => void
}>({
    mode: {
        type: "browse"
    },
    setMode: (mode: mode) => {}
});