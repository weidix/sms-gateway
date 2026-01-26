export const getStorageValue = async (/** @type {string} */ name) => {
    return new Promise((resolve, reject) => {
        try {
            const sources = [localStorage, sessionStorage];
            for (const source of sources) {
                const storedValue = source.getItem(name);
                if (storedValue !== null) {
                    resolve(JSON.parse(storedValue));
                    return;
                }
            }
            resolve(null);
        } catch (error) {
            reject(error);
        }
    });
};

export const updateStorageValue = async (
    /** @type {string} */ name,
    /** @type {any} */ value,
    /** @type {{ persistent?: boolean }} */ options = {}
) => {
    const { persistent = false } = options;

    return new Promise((resolve, reject) => {
        try {
            // Clear both storages when removing a value
            if (value === null || value === undefined) {
                sessionStorage.removeItem(name);
                localStorage.removeItem(name);
                resolve("Value removed successfully");
                return;
            }

            // Persist to both storages when remember-me is enabled to keep auth header working
            if (persistent) {
                const serialized = JSON.stringify(value);
                localStorage.setItem(name, serialized);
                sessionStorage.setItem(name, serialized);
            } else {
                localStorage.removeItem(name);
                sessionStorage.setItem(name, JSON.stringify(value));
            }
            resolve("Value updated successfully");
        } catch (error) {
            reject(error);
        }
    });
};
