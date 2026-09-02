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
///
/// 主字体(微软雅黑/苹方等)覆盖 BMP 区中文; 部分字体补充 CJK 扩展 B 区字形
/// (如 U+20000 等生僻字), 避免渲染为缺字占位符"口"。
fn setup_fonts(ctx: &eframe::egui::Context) {
    // (主中文字体候选, CJK 扩展 B 区字体候选)
    let (main_candidates, extb_candidates): (&[&str], &[&str]) = if cfg!(target_os = "windows") {
        (
            &[
                r"C:\Windows\Fonts\msyh.ttc",     // 微软雅黑 (Win7+ 默认)
                r"C:\Windows\Fonts\simsun.ttc",   // 宋体 (系统核心字体, 精简版通常保留)
                r"C:\Windows\Fonts\simhei.ttf",   // 黑体
                r"C:\Windows\Fonts\Deng.ttf",     // 等线 (Win8+ 默认 UI 字体)
                r"C:\Windows\Fonts\MingLiU.ttc",  // 細明體 (繁体系统)
                r"C:\Windows\Fonts\PMingLiU.ttc", // 新細明體 (繁体系统)
                r"C:\Windows\Fonts\msjh.ttc",     // 微軟正黑體 (繁体系统)
                r"C:\Windows\Fonts\simkai.ttf",   // 楷体
                r"C:\Windows\Fonts\simfang.ttf",  // 仿宋
            ],
            &[r"C:\Windows\Fonts\simsunb.ttf"],
        )
    } else if cfg!(target_os = "macos") {
        (
            &[
                "/System/Library/Fonts/PingFang.ttc",
                "/System/Library/Fonts/Hiragino Sans GB.ttc",
                "/System/Library/Fonts/STHeiti Light.ttc",
            ],
            &[
                "/System/Library/Fonts/Songti.ttc",
                "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
            ],
        )
    } else {
        (
            &[
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
                "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
            ],
            &[], // Noto Sans CJK 等已覆盖较广, 无专门扩展字体
        )
    };

    let mut fonts = eframe::egui::FontDefinitions::default();
    let mut loaded: Vec<String> = Vec::new();

    for (label, candidates) in [("chinese", main_candidates), ("chinese_extb", extb_candidates)] {
        let Some(path) = candidates.iter().find(|p| Path::new(p).exists()) else {
            continue;
        };
        let Ok(data) = std::fs::read(path) else { continue };
        fonts
            .font_data
            .insert(label.to_owned(), eframe::egui::FontData::from_owned(data).into());
        loaded.push(label.to_owned());
    }
    if loaded.is_empty() {
        return; // 未找到中文字体时保持默认字体
    }

    for family in [
        eframe::egui::FontFamily::Proportional,
        eframe::egui::FontFamily::Monospace,
    ] {
        let list = fonts.families.entry(family).or_default();
        list.extend(loaded.iter().cloned());
    }
    ctx.set_fonts(fonts);
}
