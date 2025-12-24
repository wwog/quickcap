# Screenshot Tool

[中文](./README.zh.md)

## Features

- Multi-monitor support
- Cross-platform support (macOS, Windows)
- Fast response (around 300ms to complete display on macOS with dual 4K monitors) (around 300ms to complete display on Windows with dual 2K monitors, even with old CPUs)
- Window awareness
- Clipboard, brush, and other common features

## Technical Implementation

Windows uses GDI for screen capture. The reason is that whether it's DXGI or GraphicCapture, GDI is faster for single-frame capture and doesn't require manual multi-monitor stitching. These two new APIs primarily serve remote and video scenarios, which are slower than GDI and more complex for most CPU-processing scenarios in the current architecture.

macOS uses the initial implementation of ScreenCaptureKit, including window enumeration, which is also part of this API to ensure balance between window acquisition and capture. Since macOS desktop space is bound to display presentation, there will be two display-sized windows as overlays, which Windows doesn't have this limitation. Additionally, I forked the screencapture-rs library, adding CGDisplayCreateImage implementation and some other features. [ForkVersion](https://github.com/wwog/screencapturekit-rs). This library implementation supports cross-compilation, while the original implementation strongly depends on the system. This would require you to have both macOS Intel and macOS ARM, and several key system versions are needed.

This project also implements enumerate_windows_cg for compatibility needs. CGWindowListCopyWindowInfo and CGDisplayCreateImage can support most macOS versions.

Early commits used wgpu for background rendering, but it doesn't support external textures. The best solution might be platform-specific implementations or Skia. However, considering complexity and platform differences, webview was chosen.

Early commits also attempted DXGI and Graphic.Capture.

Screenshot capture and a series of system calls are handled by Rust. The main latency comes from webview cold start. Subsequent optimizations could consider a service-style hidden window. It can complete 4K screen capture window opening in around 100ms.

## Execution Parameters

`--debug`: In normal mode, the window is set to screensaver-level topmost. Debug mode means you can switch to foreground.

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

Before the process exits, two messages are emitted:

- `save_image_to_folder` — indicates the image was saved successfully
- `copy_to_clipboard` — indicates the image was copied successfully

## CI/CD Workflow

The project uses GitHub Actions for automated builds and releases. The workflow configuration is located at `.github/workflows/release.yml`.

### Trigger Conditions

Create a version tag and publish:

```bash
# Create version tag
git tag v0.1.0

# Push tag (will automatically trigger build and release)
git push origin v0.1.0
```

After pushing the tag, GitHub Actions will automatically start the build process. Once completed, a new release will be created on the Releases page.

## Permissions (macOS only)

This software maintains flexibility and extensibility. When permissions are not granted, the integration side needs to handle this themselves. On first run, the screenshot API will trigger a system permission dialog (it will not appear again whether accepted or rejected). Although the dialog is triggered, the application does not poll and monitor settings or block execution. So the first time, the dialog and application startup (just a transparent window, without screenshot and window awareness data) will appear.

Solution:

macOS is designed so the dialog only appears once, but we strongly depend on permissions. Therefore, the integration side needs to display their own dialog and check permissions to block execution. I originally wanted to try implementing a dialog internally, but due to multi-language and customization requirements, I don't plan to add this later.

## Integration

To launch the window, you need the business side to start the corresponding executable binary program on the corresponding system. Note that macOS Mach-O files lose execution permissions during transfer, so they are packaged in tar.gz format for similar business layer workflows. Windows doesn't require this but is also packaged. Additionally, embedding into macOS application bundles requires consistent signatures and cannot be ignored. Mach-O files within the bundle cannot obtain permissions if not signed. Even if you authorize the upper-level application, it will not be shared with the Mach-O file. This is an effective means to prevent hackers from replacing executable files within the bundle. You can consider calling external Mach-O files (download or unpack and run, theoretically possible).
