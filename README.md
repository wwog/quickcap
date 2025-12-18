# Screenshot Program

[中文文档](./README.cn.md)

## Technical Implementation

Windows uses GDI because whether it's DXGI or GraphicCapture, single-frame capture is faster, and there's no need to stitch multiple monitors yourself. These newer APIs are primarily designed for remote desktop and video scenarios, which are slower than GDI and more complex for our current CPU-processing-based architecture.

macOS uses the initial implementation of ScreenCaptureKit, including window enumeration with the same API, to ensure a balance between window acquisition and capture. Since macOS desktop spaces are bound to displays, there are two display-sized windows as overlay layers, which Windows doesn't have this limitation. Additionally, we forked the screencapture-rs library to add CGDisplayCreateImage implementation and other features. This project also implements enumerate_windows_cg for compatibility needs. CGWindowListCopyWindowInfo and CGDisplayCreateImage support most macOS versions.

Early commits used wgpu for rendering backgrounds, but it doesn't support external textures. The best solution might be platform-specific implementations or Skia. However, considering complexity and platform differences, we chose webview.

Screenshots and system calls are handled by Rust. The main latency comes from webview cold start. Future optimization could consider using a service-style hidden window. 4K screenshot windows can be opened in about 100ms.

## Features

- Multi-monitor support
- Cross-platform support (macOS, Windows)
- Fast response (on Mac with dual 4K displays, display completes in ~300ms) (on Windows with dual 2K displays and older CPU, display completes in ~300ms)
- Window awareness

## Optimization Points

There are many optimization opportunities in this project. To achieve instant startup like WeChat, it could be changed to a service-style approach, initializing the heaviest dependency (webview) and then hiding it. Memory usage is not particularly high.

Alternatively, if complexity is not a concern, using Swift & C# for single-platform implementation is recommended. Without webview, replacing GDI on Windows, both can directly capture textures for fast rendering, achieving zero-copy. Both also support direct use of BGRA format.

Another option is using Skia, maintaining Rust for cross-platform support. Based on research (not yet attempted), Skia should support `CVImageBuffer`, `IOSurface`, and Windows `D3D11Texture`. It can also conveniently draw, frame, and export.

The best cross-platform implementation is Skia, with high cold-start efficiency and moderate complexity.

## Other

At the time of writing, cross-application communication is not yet complete. If you see this without implementation, it might use stdio, with the initiator detecting standard output to determine program execution status. Additionally, all code is implemented in the lib crate, so you can easily build dynamic libraries for calling, such as Node.js native modules or platform dynamic libraries. If the project is not completed later, it may not be completed and might use program parameters.

## Requirements

- macOS 12.3+ (internally uses screencapture for single-frame capture)
- Most Windows versions supported (uses relatively simple GDI for multi-monitor support)
