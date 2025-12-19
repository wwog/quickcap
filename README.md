# Screenshot Tool

## Technical Implementation

Windows uses GDI for screen capture. The reason is that whether it's DXGI or GraphicCapture, GDI is faster for single-frame capture and doesn't require manual multi-monitor stitching. These two new APIs primarily serve remote and video scenarios, which are slower than GDI and more complex for most CPU-processing scenarios in the current architecture.

macOS uses the initial implementation of ScreenCaptureKit, including window enumeration, which is also part of this API to ensure balance between window acquisition and capture. Since macOS desktop space is bound to display presentation, there will be two display-sized windows as overlays, which Windows doesn't have this limitation. Additionally, the screencapture-rs library was forked, adding CGDisplayCreateImage implementation and some other features. This project also implements enumerate_windows_cg for compatibility needs. CGWindowListCopyWindowInfo and CGDisplayCreateImage can support most macOS versions.

Early commits used wgpu for background rendering, but it doesn't support external textures. The best solution might be platform-specific implementations or Skia. However, considering complexity and platform differences, webview was chosen.

Early commits also attempted DXGI and Graphic.Capture.

Screenshot capture and a series of system calls are handled by Rust. The main latency comes from webview cold start. Subsequent optimizations could consider a service-style hidden window. It can complete 4K screen capture window opening in around 100ms.

## Features

- Multi-monitor support
- Cross-platform support (macOS, Windows)
- Fast response (around 300ms to complete display on macOS with dual 4K monitors) (around 300ms to complete display on Windows with dual 2K monitors, even with old CPUs)
- Window awareness
- Clipboard, brush, and other common features

## Optimization Points

There are still many optimization points in the project. To achieve WeChat's instant startup, it can be changed to a service-style approach, initializing the webview (the heaviest dependency) and then hiding it. Memory usage is not particularly high.

Alternatively, if complexity is not a concern, it's recommended to use Swift && C# for single-platform implementation. Instead of using webview, replace GDI on Windows. Both can directly capture textures for fast rendering, achieving zero-copy. Both also support direct use of BGRA.

Another option is to use Skia, keeping Rust for cross-platform. Skia has been researched but not tried. It should support `CVImageBuffer`, `IOSurface`, and Windows' `D3D11Texture`. It can also conveniently draw, frame, and other operations for final export.

The best cross-platform implementation is Skia, with very high cold start efficiency and not particularly high complexity.

## Others

At the time of writing this documentation, cross-application communication has not been completed. If it's not implemented when you see this, it may use stdio format, with the initiator detecting standard output to determine the dynamic execution of the program. Additionally, all code implementations are in the lib crate. You can also easily build dynamic libraries for calls, such as Node.js native modules or platform dynamic libraries. Note that if this functionality is used as a dynamic library, it will block the main thread. Most systems require the UI thread to be on the main thread, which is extremely difficult to solve. That is, when you call this dynamic library during your own application execution, the original application will be unresponsive until the operation is complete.

## Requirements

macOS 12.3+ (uses screencapture internally for single-frame capture)
Most Windows versions supported (needs multi-monitor support, so uses simpler GDI for acquisition)

## Communication

Unix/Linux stdio. stderr is log, stdout is data.
