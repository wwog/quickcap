# Screenshot Tool

[中文文档](./README.cn.md)

## Technical Implementation

Windows uses GDI. Although DXGI and GraphicCapture offer faster single-frame capture speeds and don't require manual multi-monitor stitching, these newer APIs are primarily designed for remote desktop and video scenarios. For our architecture's CPU-heavy processing use cases, they are slower and more complex than GDI.

macOS uses the initial implementation of ScreenCaptureKit, including window enumeration with the same API to maintain balance between window retrieval and capture. Due to macOS desktop spaces being bound to displays, two display-sized windows are used as overlay layers (Windows doesn't have this limitation). Additionally, we forked the screencapture-rs library to add CGDisplayCreateImage implementation and other features. This project also implements enumerate_windows_cg for compatibility needs. CGWindowListCopyWindowInfo and CGDisplayCreateImage support most macOS versions.

Early commits used wgpu for background rendering, but it doesn't support external textures. The optimal solution might be platform-specific implementation or Skia. However, considering complexity and platform differences, we chose webview.

Early commits also experimented with DXGI and Graphic.Capture.

Screenshot and system calls are handled by Rust. The main latency source is webview cold start. Future optimization could consider a service-style hidden window approach. A 4K screenshot window can be opened in about 100ms.

## Features

- Multi-monitor support
- Cross-platform support (macOS, Windows)
- Fast response (around 300ms to display on Mac with dual 4K monitors; around 300ms on Windows with dual 2K monitors and older CPU)
- Window awareness
- Clipboard, brush, and other common features

## Optimization Opportunities

There are many optimization opportunities in this project. To achieve instant startup like WeChat, it could be changed to a service-style approach—initializing the webview (the heaviest dependency) and keeping it hidden. Memory usage is not particularly high.

Alternatively, if complexity isn't a concern, using Swift & C# for single-platform implementation is recommended. Without webview, replacing GDI on Windows, both platforms can directly capture textures for fast rendering, achieving zero-copy. Both also support direct BGRA usage.

Another option is using Skia while keeping Rust for cross-platform support. Based on research (not yet tested), Skia should support `CVImageBuffer`, `IOSurface`, and Windows' `D3D11Texture`. It also facilitates drawing operations like frames and annotations before final export.

The best cross-platform implementation is Skia—it offers high cold-start efficiency and moderate complexity.

## Notes

At the time of writing, cross-application communication hasn't been completed. If not implemented when you read this, it might use stdio, with the caller detecting standard output to determine program execution status. Additionally, all code is implemented in the lib crate, so you can easily build dynamic libraries for invocation, such as Node.js native modules or platform dynamic libraries. Note that when used as a dynamic library, this feature blocks the main thread. Most systems require the UI thread on the main thread, which is extremely difficult to solve—meaning when you call this dynamic library from your application, the original application will be unresponsive until the operation completes.

## System Requirements

- macOS 12.3+ (internally uses ScreenCapture for single-frame capture)
- Most Windows versions supported (uses GDI for multi-monitor support due to its simplicity)

## Information exchange

unix/linux stdio. stderr is log, stdout is data.