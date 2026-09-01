//! 开发工具包: 带 UI 界面的开发工具集合
//!
//! 包含:
//! 1. 时间戳与时间格式化工具
//! 2. 字符集编码转换工具


// 仅在非调试（发布）版本时启用 "windows" 子系统
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod base64_tool;
mod checksum_tool;
mod encoding_tool;
mod hash_tool;
mod i18n;
mod theme;
mod timestamp_tool;
mod url_tool;

use std::path::Path;


fn main() -> eframe::Result {
    
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("开发工具包")
            .with_inner_size([960.0, 700.0])
            .with_min_inner_size([800.0, 560.0]),
        ..Default::default()
    };

    eframe::run_native(
        "开发工具包",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(app::ToolkitApp::new()))
        }),
    )
}

/// 加载系统中文字体, 确保中文界面正常显示
fn setup_fonts(ctx: &eframe::egui::Context) {
    let candidates: &[&str] = if cfg!(target_os = "windows") {
        &[
            r"C:\Windows\Fonts\msyh.ttc",
            r"C:\Windows\Fonts\simhei.ttf",
            r"C:\Windows\Fonts\simsun.ttc",
        ]
    } else if cfg!(target_os = "macos") {
        &[
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
        ]
    } else {
        &[
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
        ]
    };

    let Some(path) = candidates.iter().find(|p| Path::new(p).exists()) else {
        return; // 未找到中文字体时保持默认字体
    };
    let Ok(data) = std::fs::read(path) else { return };

    let mut fonts = eframe::egui::FontDefinitions::default();
    fonts
        .font_data
        .insert("chinese".to_owned(), eframe::egui::FontData::from_owned(data).into());
    for family in [
        eframe::egui::FontFamily::Proportional,
        eframe::egui::FontFamily::Monospace,
    ] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push("chinese".to_owned());
    }
    ctx.set_fonts(fonts);
}
