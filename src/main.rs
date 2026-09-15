//! 开发工具包: 带 UI 界面的开发工具集合
//!
//! 包含:
//! 1. 时间戳与时间格式化工具
//! 2. 字符集编码转换工具
//! 3. URL / Base64 编解码
//! 4. 哈希与校验计算

// 仅在非调试（发布）版本时启用 "windows" 子系统
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod base64_tool;
mod checksum_tool;
mod encoding_tool;
mod fonts;
mod hash_tool;
mod i18n;
mod theme;
mod timestamp_tool;
mod url_tool;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("开发工具包")
            .with_inner_size([1080.0, 720.0])
            .with_min_inner_size([880.0, 600.0]),
        ..Default::default()
    };

    return eframe::run_native(
        "开发工具包",
        options,
        Box::new(|cc| {
            fonts::install(&cc.egui_ctx);
            theme::apply(&cc.egui_ctx);
            return Ok(Box::new(app::ToolkitApp::new()));
        }),
    );
}
