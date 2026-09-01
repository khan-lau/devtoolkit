use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // 构建脚本 cwd 通常为 crate 根目录, 但显式使用绝对路径更健壮
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let rc_path = Path::new(&manifest_dir).join("resources.rc");
    
    // 只在 Windows 目标下嵌入应用图标
    #[cfg(target_os = "windows")]
    {
        let target = env::var("TARGET").unwrap_or_default();

        // aarch64 gnullvm 目标: embed-resource 依赖系统命令 llvm-rc,
        // 该环境通常不存在, 改用手动调用 zig 自带的 rc (drop-in rc.exe) 编译资源
        if target == "aarch64-pc-windows-gnullvm" {
            embed_icon_via_zig(&target, &rc_path);
            return;
        }

        if target == "x86_64-pc-windows-msvc"  || target == "aarch64-pc-windows-msvc" || target == "x86_64-pc-windows-gnu" || target == "aarch64-pc-windows-gnullvm" {
            println!("cargo:warning=rc_path: {}", rc_path.display());
            
            // 其余 Windows 目标 (gnu / msvc): 统一由 embed-resource 处理,
            // 它会根据工具链自动选择 rc.exe 或 windres
            match embed_resource::compile(&rc_path, embed_resource::NONE) {
                embed_resource::CompilationResult::Ok => {
                    println!("cargo:warning=icon embedded via embed-resource ({})", target);
                }
                embed_resource::CompilationResult::NotWindows => {
                    println!("cargo:warning=icon embed skipped: not a Windows target ({})", target);
                }
                embed_resource::CompilationResult::NotAttempted(msg)
                | embed_resource::CompilationResult::Failed(msg) => {
                    println!("cargo:warning=icon embed failed for {}: {}", target, msg);
                }
            }
        }
    }

    // 🍎 macOS 图标处理
    #[cfg(target_os = "macos")]
    {
        let target = env::var("TARGET").unwrap_or_default();
        if target.contains("apple-darwin") {
            embed_icon_for_macos(&manifest_dir);
        }
    }

    // 非 Windows 平台无需嵌入图标
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        println!("cargo:warning=icon embed skipped: not on Windows or macOS target");
    }
}


/// macOS 图标嵌入
#[cfg(target_os = "macos")]
fn embed_icon_for_macos(manifest_dir: &str) {
    let icon_name = "app"; // 不带扩展名
    let icns_path = Path::new(manifest_dir)
        .join("assets")
        .join(format!("{}.icns", icon_name));
    
    // 检查 .icns 文件是否存在
    if !icns_path.exists() {
        println!("cargo:warning=macOS icon not found: {}", icns_path.display());
        println!("cargo:warning=Place app.icns in assets/ directory");
        return;
    }

    // 1. 创建 Resources 目录（在 OUT_DIR 中）
    let out_dir = env::var("OUT_DIR").unwrap_or_default();
    let resources_dir = Path::new(&out_dir).join("resources");
    if let Err(e) = std::fs::create_dir_all(&resources_dir) {
        println!("cargo:warning=Failed to create resources dir: {}", e);
        return;
    }

    // 2. 复制 .icns 文件到 Resources 目录
    let dest_icns = resources_dir.join(format!("{}.icns", icon_name));
    if let Err(e) = std::fs::copy(&icns_path, &dest_icns) {
        println!("cargo:warning=Failed to copy .icns file: {}", e);
        return;
    }
    println!("cargo:warning=macOS icon copied to: {}", dest_icns.display());

    // 3. 生成 Info.plist（使用 cargo:rustc-env 注入）
    // 注意：实际的 Info.plist 应该在项目根目录，或者通过 build.rs 生成
    // 这里我们通过环境变量告诉程序图标名称
    println!("cargo:rustc-env=MACOS_ICON_NAME={}", icon_name);
    
    // 4. 告诉 Cargo 将 Resources 目录包含在 .app bundle 中
    // 这通常需要配合 cargo-bundle 或自定义脚本
    println!("cargo:warning=macOS icon processing complete");
    println!("cargo:warning=Note: For .app bundle, you need to use 'cargo bundle' or similar");
}


/// 通过 zig 的 rc 命令 (drop-in rc.exe) 编译 Windows 资源并链接进二进制
#[cfg(target_os = "windows")]
fn embed_icon_via_zig(target: &str, rc_path: &Path) {
    let out_dir = env::var("OUT_DIR").unwrap_or_default();
    println!("cargo:warning=OUT_DIR is: {}", out_dir);
    let res = format!("{}/app.res", out_dir);

    match Command::new("zig").args(["rc", "/fo", &res]).arg(rc_path).status() {
        Ok(status) if status.success() => {
            println!("cargo:rustc-link-arg-bins={}", res);
            println!("cargo:warning=icon embedded via zig rc ({})", target);
        }
        Ok(status) => println!("cargo:warning=icon embed failed via zig rc ({}): {}", target, status),
        Err(e) => println!("cargo:warning=icon embed failed via zig rc: {}", e),
    }
}
