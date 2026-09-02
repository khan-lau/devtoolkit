//! 主题与设计系统: 调色板(深色/浅色)、全局样式、通用控件
//!
//! 视觉方向: 中性灰底 + 单一靛蓝强调色, 低对比细边框, 大圆角, 克制的阴影,
//! 字体层级依靠字重(Inter 可变字重)而非颜色堆叠。

use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, CursorIcon, FontFamily, FontId, Galley, Margin, Pos2, Rect,
    Response, RichText, Sense, Stroke, StrokeKind, TextStyle, Ui, Vec2, pos2, vec2,
};

// ---------------------------------------------------------------------------
// 调色板
// ---------------------------------------------------------------------------

/// 一套完整的界面颜色
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    /// 窗口背景(内容区)
    pub bg: Color32,
    /// 侧边栏背景
    pub sidebar: Color32,
    /// 卡片底色
    pub surface: Color32,
    /// 输入框/凹陷区域底色
    pub sunken: Color32,
    /// 悬停/抬起底色
    pub raised: Color32,
    /// 细边框
    pub border: Color32,
    /// 较强边框(悬停)
    pub border_strong: Color32,
    /// 主文字
    pub text: Color32,
    /// 次要文字
    pub text_weak: Color32,
    /// 更弱文字(占位/说明)
    pub text_faint: Color32,
    /// 强调色
    pub accent: Color32,
    /// 强调色(悬停)
    pub accent_hover: Color32,
    /// 强调色(按下)
    pub accent_active: Color32,
    /// 强调色淡底(选中态背景)
    pub accent_soft: Color32,
    /// 强调色上的文字
    pub on_accent: Color32,
    /// 成功
    pub success: Color32,
    /// 警告
    pub warn: Color32,
    /// 错误
    pub error: Color32,
}

pub const DARK: Palette = Palette {
    bg: Color32::from_rgb(14, 15, 19),
    sidebar: Color32::from_rgb(18, 19, 24),
    surface: Color32::from_rgb(23, 25, 31),
    sunken: Color32::from_rgb(16, 17, 22),
    raised: Color32::from_rgb(33, 36, 44),
    border: Color32::from_rgb(38, 41, 50),
    border_strong: Color32::from_rgb(58, 62, 74),
    text: Color32::from_rgb(236, 237, 241),
    text_weak: Color32::from_rgb(142, 148, 163),
    text_faint: Color32::from_rgb(92, 98, 112),
    accent: Color32::from_rgb(124, 134, 255),
    accent_hover: Color32::from_rgb(142, 150, 255),
    accent_active: Color32::from_rgb(108, 118, 240),
    accent_soft: Color32::from_rgba_premultiplied(31, 34, 64, 255),
    on_accent: Color32::WHITE,
    success: Color32::from_rgb(52, 211, 153),
    warn: Color32::from_rgb(245, 158, 11),
    error: Color32::from_rgb(248, 113, 113),
};

pub const LIGHT: Palette = Palette {
    bg: Color32::from_rgb(245, 246, 248),
    sidebar: Color32::from_rgb(238, 240, 244),
    surface: Color32::WHITE,
    sunken: Color32::from_rgb(246, 247, 249),
    raised: Color32::from_rgb(236, 238, 243),
    border: Color32::from_rgb(226, 228, 234),
    border_strong: Color32::from_rgb(200, 204, 214),
    text: Color32::from_rgb(23, 24, 28),
    text_weak: Color32::from_rgb(107, 112, 128),
    text_faint: Color32::from_rgb(160, 165, 178),
    accent: Color32::from_rgb(79, 92, 235),
    accent_hover: Color32::from_rgb(65, 78, 222),
    accent_active: Color32::from_rgb(55, 67, 205),
    accent_soft: Color32::from_rgb(232, 235, 255),
    on_accent: Color32::WHITE,
    success: Color32::from_rgb(16, 163, 110),
    warn: Color32::from_rgb(217, 119, 6),
    error: Color32::from_rgb(220, 38, 38),
};

/// 按明暗取调色板
pub fn palette(dark: bool) -> &'static Palette {
    if dark { &DARK } else { &LIGHT }
}

/// 当前 UI 的调色板
pub fn pal(ui: &Ui) -> &'static Palette {
    palette(ui.visuals().dark_mode)
}

/// 控件圆角
const RADIUS: u8 = 8;
/// 卡片圆角
const CARD_RADIUS: u8 = 12;
/// 交互动画时长(秒)
const ANIM: f32 = 0.12;
/// Inter 字重轴
const WGHT: &str = "wght";

// ---------------------------------------------------------------------------
// 全局样式
// ---------------------------------------------------------------------------

/// 应用全局主题(深色 + 浅色, 跟随系统)
pub fn apply(ctx: &egui::Context) {
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = vec2(10.0, 10.0);
        style.spacing.button_padding = vec2(14.0, 7.0);
        style.spacing.interact_size = vec2(40.0, 32.0);
        style.spacing.window_margin = Margin::same(20);
        style.spacing.menu_margin = Margin::same(6);
        style.spacing.combo_width = 180.0;
        style.spacing.text_edit_width = 320.0;
        style.spacing.icon_width = 16.0;
        style.spacing.scroll = egui::style::ScrollStyle::floating();
        style.animation_time = ANIM;

        style.text_styles = [
            (TextStyle::Small, FontId::new(12.0, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Heading, FontId::new(22.0, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(13.5, FontFamily::Monospace)),
        ]
        .into();
    });

    ctx.set_visuals_of(egui::Theme::Dark, visuals(&DARK, true));
    ctx.set_visuals_of(egui::Theme::Light, visuals(&LIGHT, false));
}

/// 由调色板生成 egui 视觉配置
fn visuals(p: &Palette, dark: bool) -> egui::Visuals {
    let mut v = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };

    v.panel_fill = p.bg;
    v.window_fill = p.surface;
    v.window_stroke = Stroke::new(1.0, p.border);
    v.window_corner_radius = CornerRadius::same(14);
    v.window_shadow = egui::Shadow {
        offset: [0, 12],
        blur: 40,
        spread: 0,
        color: Color32::from_black_alpha(if dark { 120 } else { 40 }),
    };
    v.popup_shadow = egui::Shadow {
        offset: [0, 6],
        blur: 24,
        spread: 0,
        color: Color32::from_black_alpha(if dark { 110 } else { 30 }),
    };
    v.menu_corner_radius = CornerRadius::same(10);
    v.extreme_bg_color = p.sunken;
    v.faint_bg_color = p.sunken;
    v.code_bg_color = p.sunken;
    v.text_edit_bg_color = Some(p.sunken);
    v.override_text_color = Some(p.text);
    v.weak_text_color = Some(p.text_weak);
    v.hyperlink_color = p.accent;
    v.warn_fg_color = p.warn;
    v.error_fg_color = p.error;
    v.selection.bg_fill = p.accent.gamma_multiply(if dark { 0.45 } else { 0.28 });
    v.selection.stroke = Stroke::new(1.0, p.text);
    v.text_cursor.stroke = Stroke::new(2.0, p.accent);
    v.striped = false;

    v.widgets.noninteractive = widget(p.surface, p.surface, p.border, p.text);
    v.widgets.inactive = widget(p.sunken, p.sunken, p.border, p.text);
    v.widgets.hovered = widget(p.raised, p.raised, p.border_strong, p.text);
    v.widgets.active = widget(p.raised, p.raised, p.accent, p.text);
    v.widgets.active.bg_stroke = Stroke::new(1.5, p.accent);
    v.widgets.open = widget(p.sunken, p.sunken, p.accent, p.text);
    v
}

fn widget(bg: Color32, weak_bg: Color32, border: Color32, fg: Color32) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill: bg,
        weak_bg_fill: weak_bg,
        bg_stroke: Stroke::new(1.0, border),
        corner_radius: CornerRadius::same(RADIUS),
        fg_stroke: Stroke::new(1.0, fg),
        expansion: 0.0,
    }
}

// ---------------------------------------------------------------------------
// 文字
// ---------------------------------------------------------------------------

/// 指定字重的文本(Inter 可变字重; 回退字体忽略)
pub fn weighted(text: impl Into<String>, size: f32, weight: f32) -> RichText {
    RichText::new(text).size(size).variation(WGHT, weight)
}

/// 页面标题
pub fn page_title(ui: &mut Ui, title: &str) {
    ui.label(weighted(title, 22.0, 600.0).color(pal(ui).text));
}

/// 卡片内分区标题
pub fn section_title(ui: &mut Ui, title: &str) {
    ui.label(weighted(title, 14.0, 600.0).color(pal(ui).text));
    ui.add_space(2.0);
}

/// 字段标签(次要文字)
pub fn field_label(ui: &mut Ui, text: &str) {
    ui.label(weighted(text, 13.0, 500.0).color(pal(ui).text_weak));
}

/// 等宽展示值(结果/时间戳等)
pub fn mono_value(ui: &mut Ui, value: &str) -> Response {
    ui.label(
        RichText::new(value)
            .font(FontId::new(14.0, FontFamily::Monospace))
            .color(pal(ui).text),
    )
}

/// 状态级别
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Warn,
    Error,
}

/// 状态提示行: 圆点 + 彩色文字
pub fn status(ui: &mut Ui, level: Level, text: &str) {
    let p = pal(ui);
    let color = match level {
        Level::Warn => p.warn,
        Level::Error => p.error,
    };
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let (rect, _) = ui.allocate_exact_size(vec2(8.0, 20.0), Sense::hover());
        ui.painter().circle_filled(rect.center(), 3.0, color);
        ui.label(RichText::new(text).size(13.0).color(color));
    });
}

// ---------------------------------------------------------------------------
// 容器
// ---------------------------------------------------------------------------

/// 卡片容器: 大圆角 + 细边框 + 宽松内边距, 占满可用宽度
pub fn card<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    let p = pal(ui);
    egui::Frame::new()
        .fill(p.surface)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(CornerRadius::same(CARD_RADIUS))
        .inner_margin(Margin::symmetric(20, 18))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let ret = add_contents(ui);
            ui.allocate_space(vec2(ui.available_width(), 0.0));
            ret
        })
        .inner
}

/// 卡片之间的间距
pub fn card_gap(ui: &mut Ui) {
    ui.add_space(14.0);
}

// ---------------------------------------------------------------------------
// 输入
// ---------------------------------------------------------------------------

/// 单行输入框; `width` 为 `f32::INFINITY` 时占满剩余宽度
pub fn text_input(ui: &mut Ui, text: &mut String, hint: &str, width: f32) -> Response {
    let p = pal(ui);
    ui.add(
        egui::TextEdit::singleline(text)
            .hint_text(RichText::new(hint).color(p.text_faint))
            .margin(Margin::symmetric(12, 9))
            .desired_width(width)
            .vertical_align(egui::Align::Center),
    )
}

/// 多行文本区; `mono` 为真时使用等宽字体(适合 hex / 摘要 / 编码结果)
pub fn text_area(ui: &mut Ui, text: &mut String, hint: &str, rows: usize, mono: bool) -> Response {
    let p = pal(ui);
    let mut edit = egui::TextEdit::multiline(text)
        .hint_text(RichText::new(hint).color(p.text_faint))
        .desired_rows(rows)
        .desired_width(f32::INFINITY)
        .margin(Margin::symmetric(12, 10));
    if mono {
        edit = edit.font(TextStyle::Monospace);
    }
    ui.add(edit)
}

/// 输出区标题行: 左侧标签, 右侧复制按钮(有内容时) 与额外操作
pub fn output_header(
    ui: &mut Ui,
    label: &str,
    copy_tip: &str,
    value: &str,
    extra: impl FnOnce(&mut Ui),
) {
    ui.horizontal(|ui| {
        field_label(ui, label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !value.is_empty() {
                copy_button(ui, copy_tip, value);
            }
            extra(ui);
        });
    });
}

/// 说明文字(弱化小字, 自动换行)
pub fn hint_text(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).size(12.5).color(pal(ui).text_faint));
}

// ---------------------------------------------------------------------------
// 按钮
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum ButtonKind {
    Primary,
    Secondary,
}

/// 将富文本排版为 Galley(颜色留空, 绘制时再指定)
///
/// `max_width` 有限时超出部分以省略号截断, 否则不换行延展。
pub fn layout(ui: &Ui, text: RichText, max_width: f32) -> Arc<Galley> {
    let wrap = if max_width.is_finite() {
        egui::TextWrapMode::Truncate
    } else {
        egui::TextWrapMode::Extend
    };
    egui::WidgetText::from(text.color(Color32::PLACEHOLDER))
        .into_galley(ui, Some(wrap), max_width, TextStyle::Body)
}

/// 布局按钮文字(500 字重)
fn button_galley(ui: &Ui, text: &str, size: f32) -> Arc<Galley> {
    layout(ui, weighted(text, size, 500.0), f32::INFINITY)
}

fn paint_button(ui: &mut Ui, text: &str, kind: ButtonKind) -> Response {
    let p = pal(ui);
    let galley = button_galley(ui, text, 14.0);
    let pad = ui.spacing().button_padding;
    let size = vec2(
        galley.size().x + 2.0 * pad.x,
        (galley.size().y + 2.0 * pad.y).max(ui.spacing().interact_size.y),
    );
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let hover = ui.ctx().animate_bool_with_time(resp.id, resp.hovered(), ANIM);
        let pressed = resp.is_pointer_button_down_on();
        let (fill, stroke, fg) = match kind {
            ButtonKind::Primary => {
                let base = if pressed { p.accent_active } else { p.accent };
                (base.lerp_to_gamma(p.accent_hover, hover), Stroke::NONE, p.on_accent)
            }
            ButtonKind::Secondary => {
                let base = if pressed { p.raised } else { p.surface };
                (
                    base.lerp_to_gamma(p.raised, hover),
                    Stroke::new(1.0, p.border.lerp_to_gamma(p.border_strong, hover)),
                    p.text,
                )
            }
        };
        let painter = ui.painter();
        painter.rect(rect, CornerRadius::same(RADIUS), fill, stroke, StrokeKind::Inside);
        painter.galley(rect.center() - galley.size() / 2.0, galley, fg);
    }
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// 主要操作按钮(强调色)
pub fn primary_button(ui: &mut Ui, text: &str) -> Response {
    paint_button(ui, text, ButtonKind::Primary)
}

/// 次要按钮(描边)
pub fn button(ui: &mut Ui, text: &str) -> Response {
    paint_button(ui, text, ButtonKind::Secondary)
}

/// 图标按钮的边长
const ICON_BUTTON: f32 = 28.0;

/// 复制按钮: 图标按钮, 点击后写入剪贴板并短暂显示对勾
pub fn copy_button(ui: &mut Ui, tooltip: &str, copy_text: &str) -> bool {
    let p = pal(ui);
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(ICON_BUTTON), Sense::click());
    let clicked = resp.clicked();
    let now = ui.input(|i| i.time);
    let key = resp.id.with("copied_until");

    if clicked {
        ui.ctx().copy_text(copy_text.to_owned());
        ui.ctx().data_mut(|d| d.insert_temp(key, now + 1.2));
    }
    let copied = ui
        .ctx()
        .data_mut(|d| d.get_temp::<f64>(key))
        .is_some_and(|until| until > now);
    if copied {
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
    }

    if ui.is_rect_visible(rect) {
        let hover = ui.ctx().animate_bool_with_time(resp.id, resp.hovered(), ANIM);
        let painter = ui.painter();
        let fill = Color32::TRANSPARENT.lerp_to_gamma(p.raised, hover);
        painter.rect_filled(rect, CornerRadius::same(6), fill);
        if copied {
            draw_check(painter, rect.center(), p.success);
        } else {
            let color = p.text_weak.lerp_to_gamma(p.text, hover);
            draw_copy_icon(painter, rect.center(), color);
        }
    }
    resp.on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(tooltip);
    clicked
}

/// 复制图标: 两个错位的圆角矩形
fn draw_copy_icon(painter: &egui::Painter, center: Pos2, color: Color32) {
    let stroke = Stroke::new(1.5, color);
    let back = Rect::from_center_size(center + vec2(-2.0, -2.0), vec2(9.0, 9.0));
    let front = Rect::from_center_size(center + vec2(2.0, 2.0), vec2(9.0, 9.0));
    // 后方矩形只画露出的两条边, 形成层叠感
    painter.line_segment([back.left_bottom(), back.left_top()], stroke);
    painter.line_segment([back.left_top(), back.right_top()], stroke);
    painter.rect_stroke(front, CornerRadius::same(2), stroke, StrokeKind::Middle);
}

/// 对勾图标
fn draw_check(painter: &egui::Painter, center: Pos2, color: Color32) {
    let stroke = Stroke::new(2.0, color);
    let a = center + vec2(-5.0, 0.5);
    let b = center + vec2(-1.5, 4.0);
    let c = center + vec2(5.5, -4.0);
    painter.line_segment([a, b], stroke);
    painter.line_segment([b, c], stroke);
}

// ---------------------------------------------------------------------------
// 选择控件
// ---------------------------------------------------------------------------

/// 分段控制器: 一组互斥选项, 返回被点击的索引
pub fn segmented(ui: &mut Ui, selected: usize, options: &[&str]) -> Option<usize> {
    let p = pal(ui);
    let pad = vec2(12.0, 0.0);
    let inner_h = 28.0;
    let outer_pad = 3.0;
    let gap = 2.0;

    let galleys: Vec<Arc<Galley>> = options.iter().map(|o| button_galley(ui, o, 13.5)).collect();
    let widths: Vec<f32> = galleys.iter().map(|g| g.size().x + 2.0 * pad.x).collect();
    let total_w: f32 = widths.iter().sum::<f32>() + gap * (widths.len().saturating_sub(1)) as f32;

    let (rect, container) = ui.allocate_exact_size(
        vec2(total_w + 2.0 * outer_pad, inner_h + 2.0 * outer_pad),
        Sense::hover(),
    );
    let mut clicked = None;
    if !ui.is_rect_visible(rect) {
        return clicked;
    }

    ui.painter().rect(
        rect,
        CornerRadius::same(RADIUS + 1),
        p.sunken,
        Stroke::new(1.0, p.border),
        StrokeKind::Inside,
    );

    let mut x = rect.left() + outer_pad;
    for (i, (galley, w)) in galleys.into_iter().zip(widths).enumerate() {
        let seg = Rect::from_min_size(pos2(x, rect.top() + outer_pad), vec2(w, inner_h));
        let id = container.id.with(i);
        let resp = ui
            .interact(seg, id, Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);
        if resp.clicked() {
            clicked = Some(i);
        }
        let is_sel = i == selected;
        let sel = ui.ctx().animate_bool_with_time(id.with("sel"), is_sel, ANIM);
        let hover = ui.ctx().animate_bool_with_time(id.with("hover"), resp.hovered(), ANIM);

        let painter = ui.painter();
        if sel > 0.0 {
            let fill = p.surface.lerp_to_gamma(p.raised, 0.6).gamma_multiply(sel);
            painter.rect(
                seg,
                CornerRadius::same(RADIUS - 2),
                fill,
                Stroke::new(1.0, p.border_strong.gamma_multiply(sel)),
                StrokeKind::Inside,
            );
        }
        let fg = p
            .text_weak
            .lerp_to_gamma(p.text, sel.max(hover * 0.7));
        painter.galley(seg.center() - galley.size() / 2.0, galley, fg);
        x += w + gap;
    }
    clicked
}

/// 胶囊选项(可换行排列的单选): 返回是否被点击
pub fn chip(ui: &mut Ui, selected: bool, text: &str) -> bool {
    let p = pal(ui);
    let galley = button_galley(ui, text, 13.5);
    let size = vec2(galley.size().x + 24.0, 30.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let sel = ui.ctx().animate_bool_with_time(resp.id.with("sel"), selected, ANIM);
        let hover = ui.ctx().animate_bool_with_time(resp.id, resp.hovered(), ANIM);
        let fill = p
            .surface
            .lerp_to_gamma(p.raised, hover * (1.0 - sel))
            .lerp_to_gamma(p.accent_soft, sel);
        let border = p
            .border
            .lerp_to_gamma(p.border_strong, hover)
            .lerp_to_gamma(p.accent, sel);
        let fg = p.text_weak.lerp_to_gamma(p.text, hover).lerp_to_gamma(p.accent, sel);
        let painter = ui.painter();
        painter.rect(rect, CornerRadius::same(15), fill, Stroke::new(1.0, border), StrokeKind::Inside);
        painter.galley(rect.center() - galley.size() / 2.0, galley, fg);
    }
    resp.on_hover_cursor(CursorIcon::PointingHand).clicked()
}

/// 一行胶囊选项组: 标签 + 可换行的选项, 返回被点击的索引
pub fn chip_group(ui: &mut Ui, label: &str, selected: usize, options: &[&str]) -> Option<usize> {
    let mut clicked = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
        field_label(ui, label);
        ui.add_space(4.0);
        for (i, opt) in options.iter().enumerate() {
            if chip(ui, i == selected, opt) {
                clicked = Some(i);
            }
        }
    });
    clicked
}

/// 下拉列表里的可选项(保留 egui 原生行为, 点击后释放焦点)
pub fn menu_item(ui: &mut Ui, selected: bool, text: impl Into<egui::WidgetText>) -> bool {
    let resp = ui.add(egui::Button::selectable(selected, text).min_size(vec2(ui.available_width(), 30.0)));
    if resp.clicked() {
        ui.ctx().memory_mut(|m| m.surrender_focus(resp.id));
        true
    } else {
        false
    }
}

// ---------------------------------------------------------------------------
// 复合行
// ---------------------------------------------------------------------------

/// 结果行: 标签 + 等宽值 + 复制按钮(在 3 列 Grid 中使用)
pub fn result_row(ui: &mut Ui, label: &str, value: &str, copy_tip: &str) {
    field_label(ui, label);
    mono_value(ui, value);
    copy_button(ui, copy_tip, value);
    ui.end_row();
}

/// 结果区块: 淡底容器, 内部为 3 列 Grid(标签 / 值 / 操作)
pub fn result_block<R>(ui: &mut Ui, id: &str, add_rows: impl FnOnce(&mut Ui) -> R) -> R {
    let p = pal(ui);
    egui::Frame::new()
        .fill(p.sunken)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(CornerRadius::same(RADIUS + 2))
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            egui::Grid::new(id)
                .num_columns(3)
                .spacing([16.0, 6.0])
                .show(ui, add_rows)
                .inner
        })
        .inner
}
