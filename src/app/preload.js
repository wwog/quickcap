window.app = {
    exit: () => window.ipc.postMessage('str:exit'),
    getImage: async () => {
        const response = await fetch('quickcap://bg');
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
    }
}