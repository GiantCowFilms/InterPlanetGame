import React, { useEffect, useState } from "react"

export const useStorageState = (key: string) => {
    // getItem returns null for missing keys, so we convert it to undefined
    const [value,setValue] = useState(window.localStorage.getItem(key) ?? undefined);
    return [value,((value) => {
        setValue(prevValue => {
            const newValue = typeof value === 'function' ? value(prevValue) : value;
            if (newValue === undefined) {
                window.localStorage.removeItem(key);
            } else {
                window.localStorage.setItem(key,newValue);
            }
            return newValue;
        });
    }) as typeof setValue] as const;
}

type serverActionStatus = "init"|"done";
/**
 * Allows us to consume the result of some action that needs to be completed on the server
 * declaritively. This is useful to avoid showing elements of the UI (such as the game window)
 * until all the proper setup actions have been completed
 */
export const useSeverActionResult = (action: (complete: () => void) => ReturnType<React.EffectCallback>) => {
    const [status,setStatus] = useState<serverActionStatus>("init");
    useEffect(() => {
        const cleanup = action(() => {
            setStatus("done");
        });
        return cleanup;
    },[action,setStatus]);
    return status;
}