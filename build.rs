use std::env;

fn main() {
    // 只在 Windows 处理资源
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let version = env::var("CARGO_PKG_VERSION")
        .expect("CARGO_PKG_VERSION not set");

    let version_parts: Vec<u32> = version
        .split('.')
        .map(|s| s.parse::<u32>().unwrap_or(0))
        .chain(std::iter::repeat(0))
        .take(4)
        .collect();

    // 将版本号转换为 u64 格式: (major << 48) | (minor << 32) | (patch << 16) | build
    let version_u64 = ((version_parts[0] as u64) << 48)
        | ((version_parts[1] as u64) << 32)
        | ((version_parts[2] as u64) << 16)
        | (version_parts[3] as u64);

    let mut res = winres::WindowsResource::new();
    
    // 设置图标
    res.set_icon("icons/app.ico");
    
    // 设置版本信息
    res.set_version_info(winres::VersionInfo::FILEVERSION, version_u64);
    res.set_version_info(winres::VersionInfo::PRODUCTVERSION, version_u64);
    
    // 设置字符串信息
    res.set("CompanyName", "QuickCap");
    res.set("FileDescription", "QuickCap Screenshot Application");
    res.set("ProductName", "QuickCap");
    res.set("ProductVersion", &version);
    res.set("FileVersion", &version);
    res.set("OriginalFilename", "quickcap.exe");
    res.set("LegalCopyright", "Copyright 2025 QuickCap");
    res.set("Comments", "QuickCap is a cross-platform screenshot application, a pure green software.");
    
    
    // 编译资源
    res.compile().expect("Failed to compile Windows resources");
}
