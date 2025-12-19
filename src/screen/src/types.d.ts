interface App {
    exit: () => void;
    getImage: () => Promise<{
        width: number;
        height: number;
        arrayBuffer: ArrayBuffer;
    }>;
    copyToClipboard: (imageData: any) => Promise<Response>;
    saveImageToFolder: (imageData: any) => Promise<Response>;
    getWindows: () => Promise<any>;
    notify: (method: any, params?: {}) => void;
}


declare global {
    interface Window {
        app: App;
    }
}