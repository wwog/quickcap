# 截图程序 

[English](./README.md)

## 特性

 -  多显示器支持
 -  跨平台支持(macos,windows)
 -  快速响应 (双4k显示器的mac下，300ms左右完成展示)(双2k显示器的windows，cpu很老在300ms左右完成展示)
 -  窗口感知
 -  粘贴板，画笔等常见功能。

## 技术实现

Windows使用GDI进行截屏，原因无论是DXGI还是GraphicCapture，GDI在单帧截取速度更快，并且不需要自己拼接多显示器，因为这两个新的api主要服务远程和视频，对于目前架构的大部分cpu处理的场景慢于GDI且复杂度高。

Macos使用ScreenCaptureKit的最初实现完成，包括窗口枚举也是这套api，以确保窗口获取和截取保持平衡，由于Macos桌面空间与显示器呈现绑定状态，所以会有两个显示器尺寸的窗口作为蒙层，windows没有这个限制。另外，我fork了screencapture-rs这个库，追加了CGDisplayCreateImage的实现和一些其他功能。[ForkVersion](https://github.com/wwog/screencapturekit-rs),在此库实现中支持了交叉编译，原有实现强依赖系统。会导致你需要macos intel，也需要macos arm,且几个关节节点的系统版本都需要。

本项目也实现了enumerate_windows_cg用于需要兼容性的实现。CGWindowListCopyWindowInfo和CGDisplayCreateImage可以支持大部分macos。

早期提交采用了wgpu进行绘制背景，但并不支持外部纹理，可能最佳方案是平台特定实现或者skia。但考虑复杂度和平台差异选用webview。

早期提交也尝试了DXGI和Craphic.Capture。

截图和一系列系统调用由rust负责。主要的延迟来源于webview的冷启动，后续优化可以考虑为服务式隐藏窗口。可以在100ms左右完成4k的截屏窗口打开。

## 执行参数

--debug "正常模式下，窗口会设置为屏保级别置顶，debug意味着可以切换前台"

## 优化点

项目的优化点还是比较多，如果要达到微信的秒启动，可以更改为服务式，初始化好webview这个最重的依赖然后隐藏。内存使用并不算高。

其次如果不考虑复杂度，推荐使用swift && c#来单端实现。不去使用webview，windows替换掉GDI，他们都可以直接截取纹理，进行快速渲染，可以达到0拷贝。并且都支持bgra的直接使用。

还有一种选择是使用skia，保持rust进行跨平台，skia根据调研但并未尝试，应该是支持`CVImageBuffer``IOSurface`和windows的`D3D11Texture`。也可以方便的绘图，画框等操作到最后导出。

跨平台的最佳实现就是skia，冷启动的效率也会非常高，复杂度也不算特别高。


## 其他

文档编写时，还未完成跨应用的通信，如果看到时没有实现，那可能采用的stdio的形式，由发起方进行标准输出的检测来判定程序执行的动态。另外，所有代码实现在lib crate,你也可以轻松的构建动态库来进行调用，例如nodejs的native module。或者平台动态库。需要注意的是，此功能如果作为动态库，会阻塞主线程，大多数系统要求ui线程位于主线程，此处极难解决，也就是在你本身应用执行时调用此动态库，原有的应用会在操作完成前无响应。

## 运行要求

macos12.3以上 (内部使用screencapture截取单帧)
windows大部分支持 (需要支持多显示器，所以使用了较为简单的GDI进行获取)


## 通信

unix/linux stdio. stderr is log, stdout is data.

进程结束前会发起两个消息

1.save_image_to_folder 代表保存成功,附带路径

2.copy_to_clipboard    代表拷贝成功,附带长宽


## CI/CD 工作流

项目使用 GitHub Actions 进行自动化构建和发布，配置文件位于 `.github/workflows/release.yml`。

### 触发条件

创建版本标签并发布：

```bash
# 创建版本标签
git tag v0.1.0

# 推送标签（将自动触发构建和发布）
git push origin v0.1.0
```

推送标签后，GitHub Actions 会自动开始构建，完成后会在 Releases 页面创建新的发布版本。

## 权限 (仅 macos)

本软件保持灵活度和扩展度，没有权限的情况需要接入端自行控制。如果首次运行,截图API会调起系统权限弹窗（无论接受拒绝都不会再次弹出）。虽然调起弹窗，但应用没有轮询监控设置并阻塞运行。所以会出现第一次弹窗和应用启动(就透明窗口，并没有截图和窗口感知数据)。

解决方案：

macos设计弹窗只出现一次，但是我们强依赖权限。所以接入业务端需要自行弹窗和检测权限进行阻断运行。本身想尝试内部实现调用dialog，但因多语言和客制化需求我并不考虑后续追加。

## 接入

启动窗口所以你需要业务端在对应系统启动对应可执行二进制程序。需要注意，Macos的Mach-o文件在传输时会丢失执行权限，所以都进行了tar.gz格式的封包,为了业务层流程相近。windows不必要但是也进行了封包。其次，Macos下嵌入应用包体需要签名一致，不能忽略，对于包体内的Mach-o文件如果不签名是无法获取权限的。及时你授权了上层应用，也不会共享给Mach-o文件。这是为了避免黑客替换包体内执行文件的有效手段。可以考虑调用外部Mach-o（下载或者解包运行，理论上可以）。