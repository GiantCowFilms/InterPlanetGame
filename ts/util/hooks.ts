import { useState } from "react"

export const useStorageState = (key: string) => {
    // getItem returns null for missing keys, so we convert it to undefined
    const [value,setValue] = useState(window.localStorage.getItem(key) ?? undefined);
    return [value,((value) => {
        setValue(prevValue => {
            const newValue = typeof value === 'function' ? value(prevValue) : value;
            window.localStorage.setItem(key,newValue);
            return newValue;
        });
    }) as typeof setValue] as const;
}