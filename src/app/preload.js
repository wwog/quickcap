const app = {
    exit: () => {
        window.ipc.postMessage('exit')
    },
    getImage: async () => {
        const response = await fetch('/bg');
        const width = parseInt(response.headers.get('x-frame-width') || '0');
        const height = parseInt(response.headers.get('x-frame-height') || '0');
        // response.headers.forEach((value, key) => {
        //     console.log(`${key}: ${value}`);
        // });
        const arrayBuffer = await response.arrayBuffer();
        return {
            width,
            height,
            arrayBuffer
        }
    },
    copyToClipboard: async (imageData) => {
        return await fetch("/copy", {
            method: "POST",
            headers: {
                'x-frame-width': imageData.width,
                'x-frame-height': imageData.height,
            },
            body: imageData.data,
        })
    },
    saveImageToFolder: async (imageData) => {
        return await fetch("/save", {
            method: "POST",
            headers: {
                'x-frame-width': imageData.width,
                'x-frame-height': imageData.height,
            },
            body: imageData.data,
        })
    },
    getWindows: async () => {
        const response = await fetch('/windows');
        const windows = await response.json();
        return windows;
    },
    notify: (method, params = {}) => {
        window.ipc.postMessage(JSON.stringify({
            type: 'notify',
            method,
            params
        }));
    },
    isDebug: undefined,
}

window.app = app;