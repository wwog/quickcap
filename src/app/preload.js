window.app = {
    exit: () => window.ipc.postMessage('exit'),
    getImage: async () => {
        const response = await fetch('/bg');
        const width = parseInt(response.headers.get('x-frame-width') || '0');
        const height = parseInt(response.headers.get('x-frame-height') || '0');
        response.headers.forEach((value, key) => {
            console.log(`${key}: ${value}`);
        });
        const arrayBuffer = await response.arrayBuffer();
        return {
            width,
            height,
            arrayBuffer
        }
    },
    copyToClipboard: async (imageData) => {
        // 直接传递二进制数据，使用更高效的方式
       /*  const uint8Array = new Uint8Array(imageData);
        // 将二进制数据转换为base64字符串，确保数据完整性
        const base64 = btoa(String.fromCharCode(...uint8Array)); */
        window.ipc.postMessage('clipboard:base64:' + imageData.replace('data:image/png;base64,', ''));
    },
    saveImageToFolder: async (imageData) => {
        window.ipc.postMessage('save:' + imageData.replace('data:image/png;base64,', ''));
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
    }
}