import React from "react";

export type mode = {
    type: "game",
    game: any,
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