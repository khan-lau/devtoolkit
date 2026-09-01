//! 主题与样式: 全局深色主题、颜色常量、卡片容器

use eframe::egui;

/// 背景色(窗口最外层)
pub const BG: egui::Color32 = egui::Color32::from_rgb(22, 26, 34);
/// 面板底色
pub const PANEL: egui::Color32 = egui::Color32::from_rgb(27, 32, 41);
/// 卡片底色
pub const CARD: egui::Color32 = egui::Color32::from_rgb(33, 40, 55);
/// 悬停底色
pub const CARD_HOVER: egui::Color32 = egui::Color32::from_rgb(40, 48, 65);
/// 边框色
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(48, 57, 74);
/// 强调色(选中/按钮)
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(91, 140, 255);
/// 强调色(按下)
pub const ACCENT_DARK: egui::Color32 = egui::Color32::from_rgb(61, 110, 232);
/// 主文字色
pub const TEXT: egui::Color32 = egui::Color32::from_rgb(232, 236, 244);
/// 错误色
pub const ERROR: egui::Color32 = egui::Color32::from_rgb(255, 107, 107);
/// 警告色
pub const WARN: egui::Color32 = egui::Color32::from_rgb(230, 162, 60);

/// 应用全局主题
pub fn apply(ctx: &egui::Context) {
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(14.0, 7.0);
        style.spacing.interact_size = egui::vec2(0.0, 32.0);
        style.spacing.combo_width = 160.0;
        style.spacing.text_edit_width = 360.0;
    });

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = PANEL;
    visuals.window_fill = CARD;
    visuals.extreme_bg_color = BG;
    visuals.override_text_color = Some(TEXT);
    visuals.faint_bg_color = egui::Color32::from_rgb(40, 48, 65);
    // 选中态(标签页/选项按钮/文本选择): 亮蓝背景 + 浅色文字, 保证对比清晰
    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = egui::Stroke::new(1.0, TEXT);

    visuals.widgets.noninteractive = widget_visuals(CARD, BORDER, TEXT);
    visuals.widgets.inactive = widget_visuals(CARD_HOVER, BORDER, TEXT);
    visuals.widgets.hovered = widget_visuals(CARD_HOVER, ACCENT, TEXT);
    visuals.widgets.active = widget_visuals(ACCENT_DARK, ACCENT, TEXT);
    visuals.widgets.open = widget_visuals(CARD, ACCENT, TEXT);

    let light_visuals = visuals.clone();
    ctx.set_visuals_of(egui::Theme::Dark, visuals);
    ctx.set_visuals_of(egui::Theme::Light, light_visuals);
}

/// 构造统一风格的控件视觉
fn widget_visuals(
    bg: egui::Color32,
    border: egui::Color32,
    fg: egui::Color32,
) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill: bg,
        weak_bg_fill: CARD,
        bg_stroke: egui::Stroke::new(1.0, border),
        corner_radius: egui::CornerRadius::same(6),
        fg_stroke: egui::Stroke::new(1.0, fg),
        expansion: 0.0,
    }
}

/// 卡片容器: 圆角 + 边框 + 内边距的分组区域
///
/// 内容结束后占满剩余可用宽度, 保证同一界面内所有卡片宽度一致。
pub fn card<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(CARD)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let ret = add_contents(ui);
            // 占满剩余宽度, 确保 frame 绘制宽度一致
            ui.allocate_space(egui::vec2(ui.available_width(), 0.0));
            ret
        })
        .inner
}

/// 分区标题: 左侧强调色竖条 + 标题文字
pub fn section_title(ui: &mut egui::Ui, title: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(3.0, 16.0),
            egui::Sense::hover(),
        );
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(2), ACCENT);
        ui.label(egui::RichText::new(title).strong().size(15.0));
    });
    ui.add_space(4.0);
}

/// 复制按钮: 点击后将内容写入剪贴板
pub fn copy_button(ui: &mut egui::Ui, text: &str, copy_text: &str) -> bool {
    if ui.small_button(text).clicked() {
        ui.ctx().copy_text(copy_text.to_string());
        true
    } else {
        false
    }
}

/// 可选按钮(单选框/标签页)
///
/// 始终绘制背景与边框, 避免未选中项在获得焦点/悬停时才"突现"背景块,
/// 造成文字相对按钮位移的观感。点击后释放键盘焦点, 不再持续显示聚焦视觉。
pub fn selectable_label(
    ui: &mut egui::Ui,
    selected: bool,
    text: impl Into<egui::WidgetText>,
) -> bool {
    let resp = ui.add(egui::Button::new(text).selected(selected));
    let clicked = resp.clicked();
    if clicked {
        ui.ctx().memory_mut(|m| m.surrender_focus(resp.id));
    }
    clicked
}
