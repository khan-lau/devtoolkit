//! 主应用框架: 左侧导航栏(品牌 / 工具列表 / 语言 / 主题 / 关于) + 右侧内容区

use eframe::egui::{
    self, Align, Color32, CornerRadius, CursorIcon, FontFamily, FontId, Layout, Margin, Pos2,
    Rect, RichText, Sense, Stroke, StrokeKind, Vec2, vec2,
};

use crate::base64_tool::Base64Tool;
use crate::checksum_tool::ChecksumTool;
use crate::encoding_tool::EncodingTool;
use crate::hash_tool::HashTool;
use crate::i18n::{I18n, Texts};
use crate::theme;
use crate::timestamp_tool::TimestampTool;
use crate::url_tool::UrlTool;

/// 侧边栏宽度
const SIDEBAR_WIDTH: f32 = 232.0;

/// 工具页
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Timestamp,
    Encoding,
    Url,
    Base64,
    Hash,
    Checksum,
}

impl Tab {
    const ALL: [Tab; 6] = [
        Tab::Timestamp,
        Tab::Encoding,
        Tab::Url,
        Tab::Base64,
        Tab::Hash,
        Tab::Checksum,
    ];

    /// 导航徽标上的简写
    fn mark(self) -> &'static str {
        match self {
            Tab::Timestamp => "TS",
            Tab::Encoding => "Aa",
            Tab::Url => "%",
            Tab::Base64 => "64",
            Tab::Hash => "#",
            Tab::Checksum => "✓",
        }
    }

    /// 导航文字
    fn label(self, t: &Texts) -> &str {
        match self {
            Tab::Timestamp => &t.tab_timestamp,
            Tab::Encoding => &t.tab_encoding,
            Tab::Url => &t.tab_url,
            Tab::Base64 => &t.tab_base64,
            Tab::Hash => &t.tab_hash,
            Tab::Checksum => &t.tab_checksum,
        }
    }

    /// 页面标题
    fn title(self, t: &Texts) -> &str {
        match self {
            Tab::Timestamp => &t.ts_title,
            Tab::Encoding => &t.enc_title,
            Tab::Url => &t.url_title,
            Tab::Base64 => &t.b64_title,
            Tab::Hash => &t.hash_title,
            Tab::Checksum => &t.checksum_title,
        }
    }
}

/// 主应用: 持有各个工具的界面状态
pub struct ToolkitApp {
    i18n: I18n,
    tab: Tab,
    show_about: bool,
    timestamp: TimestampTool,
    encoding: EncodingTool,
    url: UrlTool,
    base64: Base64Tool,
    hash: HashTool,
    checksum: ChecksumTool,
}

impl ToolkitApp {
    pub fn new() -> Self {
        let i18n = I18n::load();
        let t = i18n.texts().clone();
        Self {
            i18n,
            tab: Tab::Timestamp,
            show_about: false,
            timestamp: TimestampTool::new(t.clone()),
            encoding: EncodingTool::new(t.clone()),
            url: UrlTool::new(t.clone()),
            base64: Base64Tool::new(t.clone()),
            hash: HashTool::new(t.clone()),
            checksum: ChecksumTool::new(t),
        }
    }
}

impl eframe::App for ToolkitApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // 时间戳界面需要每秒自动刷新当前时间(不依赖鼠标移动等事件)
        if self.tab == Tab::Timestamp {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs(1));
        }

        // 克隆当前语言文案与语言列表, 避免与 self 的借用冲突
        let mut t = self.i18n.texts().clone();
        let prev_lang = self.i18n.current;
        let mut lang = self.i18n.current;
        let lang_names: Vec<String> = self
            .i18n
            .langs
            .iter()
            .map(|l| l.native_name.clone())
            .collect();

        let p = theme::pal(ui);

        // 左侧导航栏
        egui::Panel::left("sidebar")
            .exact_size(SIDEBAR_WIDTH)
            .resizable(false)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(p.sidebar)
                    .inner_margin(Margin {
                        left: 14,
                        right: 14,
                        top: 18,
                        bottom: 14,
                    }),
            )
            .show(ui, |ui| {
                // 右侧分隔线
                let r = ui.max_rect();
                let x = r.right() + 14.0;
                ui.painter().vline(x, r.y_range(), Stroke::new(1.0, p.border));
                self.sidebar(ui, &t, &lang_names, &mut lang);
            });

        // 语言切换时同步文案与窗口标题
        if lang != prev_lang {
            self.i18n.current = lang;
            t = self.i18n.texts().clone();
            self.timestamp.set_lang(t.clone());
            self.encoding.set_lang(t.clone());
            self.url.set_lang(t.clone());
            self.base64.set_lang(t.clone());
            self.hash.set_lang(t.clone());
            self.checksum.set_lang(t.clone());
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Title(t.app_title.clone()));
        }

        // 内容区
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(p.bg))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Frame::new()
                            .inner_margin(Margin {
                                left: 32,
                                right: 32,
                                top: 26,
                                bottom: 24,
                            })
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                theme::page_title(ui, self.tab.title(&t));
                                ui.add_space(14.0);
                                match self.tab {
                                    Tab::Timestamp => self.timestamp.ui(ui),
                                    Tab::Encoding => self.encoding.ui(ui),
                                    Tab::Url => self.url.ui(ui),
                                    Tab::Base64 => self.base64.ui(ui),
                                    Tab::Hash => self.hash.ui(ui),
                                    Tab::Checksum => self.checksum.ui(ui),
                                }
                            });
                    });
            });

        // 关于窗口
        if self.show_about {
            self.about_window(ui.ctx(), &t);
        }
    }
}

impl ToolkitApp {
    /// 侧边栏内容
    fn sidebar(&mut self, ui: &mut egui::Ui, t: &Texts, lang_names: &[String], lang: &mut usize) {
        let p = theme::pal(ui);

        // 品牌
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            ui.add_space(2.0);
            brand_mark(ui, 30.0);
            ui.label(theme::weighted(&t.app_title, 15.0, 600.0).color(p.text));
        });
        ui.add_space(22.0);

        // 底部: 语言 / 主题 / 关于 (从下往上排)
        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let combo_w = ui.available_width() - 32.0 - 8.0;
                egui::ComboBox::from_id_salt("lang_select")
                    .selected_text(RichText::new(&lang_names[*lang]).size(13.0))
                    .width(combo_w)
                    .show_ui(ui, |ui| {
                        for (idx, name) in lang_names.iter().enumerate() {
                            if theme::menu_item(ui, *lang == idx, RichText::new(name).size(13.0)) {
                                *lang = idx;
                            }
                        }
                    });
                theme_toggle(ui);
            });
            if sidebar_button(ui, &t.btn_about) {
                self.show_about = !self.show_about;
            }

            // 中间: 工具导航(占据剩余空间, 从上往下排)
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                for tab in Tab::ALL {
                    if nav_item(ui, self.tab == tab, tab.mark(), tab.label(t)) {
                        self.tab = tab;
                    }
                }
            });
        });
    }

    /// 关于窗口: 版本/作者/联系方式等信息
    fn about_window(&mut self, ctx: &egui::Context, t: &Texts) {
        let (author_name, author_contact) = cargo_author();
        let mut open = self.show_about;
        let mut close_clicked = false;

        egui::Window::new(theme::weighted(&t.about_title, 15.0, 600.0))
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .default_width(400.0)
            .open(&mut open)
            .show(ctx, |ui| {
                let p = theme::pal(ui);
                ui.style_mut().interaction.selectable_labels = false;
                ui.set_width(400.0);
                ui.add_space(6.0);

                // 品牌 + 版本
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 14.0;
                    brand_mark(ui, 48.0);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.label(theme::weighted(&t.app_title, 18.0, 600.0).color(p.text));
                        ui.horizontal(|ui| {
                            version_pill(ui, &format!("v{}", env!("CARGO_PKG_VERSION")));
                            ui.label(
                                RichText::new(env!("CARGO_PKG_LICENSE"))
                                    .size(12.5)
                                    .color(p.text_weak),
                            );
                        });
                    });
                });
                ui.add_space(12.0);
                ui.label(RichText::new(&t.about_desc).size(13.0).color(p.text_weak));
                ui.add_space(14.0);

                // 详情
                theme::result_block(ui, "about_grid", |ui| {
                    ui.style_mut().interaction.selectable_labels = true;
                    about_row(ui, &t.about_version, env!("CARGO_PKG_VERSION"), true);
                    about_row(ui, &t.about_author, &author_name, false);
                    about_row(ui, &t.about_contact, &author_contact, false);
                    about_row(ui, &t.about_license, env!("CARGO_PKG_LICENSE"), false);
                    about_row(ui, &t.about_built, &build_info(), true);
                });

                ui.add_space(14.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::primary_button(ui, &t.btn_close).clicked() {
                        close_clicked = true;
                    }
                });
            });

        if close_clicked {
            open = false;
        }
        self.show_about = open;
    }
}

/// 关于窗口中的一行(标签 / 值 / 占位)
fn about_row(ui: &mut egui::Ui, label: &str, value: &str, mono: bool) {
    let p = theme::pal(ui);
    theme::field_label(ui, label);
    if mono {
        theme::mono_value(ui, value);
    } else {
        ui.label(RichText::new(value).size(13.5).color(p.text));
    }
    ui.label("");
    ui.end_row();
}

/// 品牌图标: 圆角方块 + 等宽 "{}" 记号
fn brand_mark(ui: &mut egui::Ui, size: f32) {
    let p = theme::pal(ui);
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same((size * 0.3) as u8), p.accent);
    painter.text(
        rect.center() + vec2(0.0, -size * 0.03),
        egui::Align2::CENTER_CENTER,
        "{}",
        FontId::new(size * 0.46, FontFamily::Monospace),
        p.on_accent,
    );
}

/// 版本号胶囊
fn version_pill(ui: &mut egui::Ui, text: &str) {
    let p = theme::pal(ui);
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        FontId::new(12.0, FontFamily::Monospace),
        p.accent,
    );
    let (rect, _) = ui.allocate_exact_size(galley.size() + vec2(16.0, 6.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(10), p.accent_soft);
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, p.accent);
}

/// 导航项: 徽标 + 文字, 选中时使用强调色淡底
fn nav_item(ui: &mut egui::Ui, selected: bool, mark: &str, title: &str) -> bool {
    let p = theme::pal(ui);
    let height = 38.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click());

    if ui.is_rect_visible(rect) {
        let sel = ui
            .ctx()
            .animate_bool_with_time(resp.id.with("sel"), selected, 0.14);
        let hover = ui
            .ctx()
            .animate_bool_with_time(resp.id, resp.hovered(), 0.12);

        let fill = Color32::TRANSPARENT
            .lerp_to_gamma(p.raised, hover * (1.0 - sel))
            .lerp_to_gamma(p.accent_soft, sel);
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(8), fill);

        // 徽标
        let badge = Rect::from_center_size(
            Pos2::new(rect.left() + 10.0 + 12.0, rect.center().y),
            Vec2::splat(24.0),
        );
        let badge_fill = p.raised.lerp_to_gamma(p.accent, sel);
        let badge_fg = p.text_weak.lerp_to_gamma(p.on_accent, sel);
        painter.rect(
            badge,
            CornerRadius::same(7),
            badge_fill,
            Stroke::new(1.0, p.border.gamma_multiply(1.0 - sel)),
            StrokeKind::Inside,
        );
        let mark_galley = theme::layout(ui, theme::weighted(mark, 10.5, 700.0), f32::INFINITY);
        painter.galley(badge.center() - mark_galley.size() / 2.0, mark_galley, badge_fg);

        // 文字
        let fg = p
            .text_weak
            .lerp_to_gamma(p.text, hover)
            .lerp_to_gamma(p.accent, sel);
        let galley = theme::layout(ui, theme::weighted(title, 14.0, 500.0), rect.width() - 56.0);
        painter.galley(
            Pos2::new(badge.right() + 10.0, rect.center().y - galley.size().y / 2.0),
            galley,
            fg,
        );
    }
    resp.on_hover_cursor(CursorIcon::PointingHand).clicked()
}

/// 侧边栏底部的通栏按钮
fn sidebar_button(ui: &mut egui::Ui, text: &str) -> bool {
    let p = theme::pal(ui);
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let hover = ui
            .ctx()
            .animate_bool_with_time(resp.id, resp.hovered(), 0.12);
        let painter = ui.painter();
        painter.rect(
            rect,
            CornerRadius::same(8),
            p.sidebar.lerp_to_gamma(p.raised, hover),
            Stroke::new(1.0, p.border.lerp_to_gamma(p.border_strong, hover)),
            StrokeKind::Inside,
        );
        let galley = theme::layout(ui, theme::weighted(text, 13.5, 500.0), f32::INFINITY);
        painter.galley(
            rect.center() - galley.size() / 2.0,
            galley,
            p.text_weak.lerp_to_gamma(p.text, hover),
        );
    }
    resp.on_hover_cursor(CursorIcon::PointingHand).clicked()
}

/// 主题切换按钮: 深色显示太阳, 浅色显示月亮
fn theme_toggle(ui: &mut egui::Ui) {
    let p = theme::pal(ui);
    let dark = ui.visuals().dark_mode;
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let hover = ui
            .ctx()
            .animate_bool_with_time(resp.id, resp.hovered(), 0.12);
        let painter = ui.painter();
        painter.rect(
            rect,
            CornerRadius::same(8),
            p.sidebar.lerp_to_gamma(p.raised, hover),
            Stroke::new(1.0, p.border.lerp_to_gamma(p.border_strong, hover)),
            StrokeKind::Inside,
        );
        let color = p.text_weak.lerp_to_gamma(p.text, hover);
        let c = rect.center();
        if dark {
            // 太阳: 圆心 + 8 条短射线
            painter.circle_stroke(c, 3.6, Stroke::new(1.5, color));
            for i in 0..8 {
                let a = std::f32::consts::TAU * i as f32 / 8.0;
                let dir = vec2(a.cos(), a.sin());
                painter.line_segment(
                    [c + dir * 6.0, c + dir * 8.0],
                    Stroke::new(1.5, color),
                );
            }
        } else {
            // 月牙: 大圆减去偏移的小圆(用背景色覆盖)
            let bg = p.sidebar.lerp_to_gamma(p.raised, hover);
            painter.circle_filled(c, 6.5, color);
            painter.circle_filled(c + vec2(3.2, -2.6), 5.6, bg);
        }
    }
    if resp
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
    {
        ui.ctx().set_theme(if dark {
            egui::ThemePreference::Light
        } else {
            egui::ThemePreference::Dark
        });
    }
}

/// 编译信息: 目标平台 + 构建类型
fn build_info() -> String {
    let profile = if cfg!(debug_assertions) { "debug" } else { "release" };
    format!("{} · {profile}", option_env!("BUILD_TARGET").unwrap_or("unknown"))
}

/// 从 Cargo.toml 的 authors 字段解析 (作者名, 联系方式)
fn cargo_author() -> (String, String) {
    // 格式: "Name <email>", 多个作者以逗号分隔, 只取第一个
    let author = env!("CARGO_PKG_AUTHORS").split(',').next().unwrap_or("").trim();
    if let Some(open) = author.find('<') {
        let name = author[..open].trim().to_string();
        let email = author[open + 1..]
            .trim_end_matches('>')
            .trim()
            .to_string();
        (name, email)
    } else {
        (author.to_string(), String::new())
    }
}
