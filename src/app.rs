//! 主应用框架: 顶部工具条(标题/标签页/语言/关于) + 内容区

use eframe::egui;

use crate::base64_tool::Base64Tool;
use crate::checksum_tool::ChecksumTool;
use crate::encoding_tool::EncodingTool;
use crate::hash_tool::HashTool;
use crate::i18n::{I18n, Texts};
use crate::theme;
use crate::timestamp_tool::TimestampTool;
use crate::url_tool::UrlTool;

/// 主应用: 持有各个工具的界面状态
pub struct ToolkitApp {
    i18n: I18n,
    tab: usize,
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
            tab: 0,
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
        if self.tab == 0 {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs(1));
        }

        // 克隆当前语言文案与语言列表, 避免与 self 的借用冲突
        let mut t = self.i18n.texts().clone();
        let prev_current = self.i18n.current;
        let mut current = self.i18n.current;
        let lang_names: Vec<String> = self
            .i18n
            .langs
            .iter()
            .map(|l| l.native_name.clone())
            .collect();

        // 顶部工具条
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                // 与内容区的内边距对齐
                ui.add_space(8.0);
                ui.label(egui::RichText::new(&t.app_title).strong().size(16.0));
                ui.separator();
                if theme::selectable_label(ui, self.tab == 0, &t.tab_timestamp) {
                    self.tab = 0;
                }
                if theme::selectable_label(ui, self.tab == 1, &t.tab_encoding) {
                    self.tab = 1;
                }
                if theme::selectable_label(ui, self.tab == 2, &t.tab_url) {
                    self.tab = 2;
                }
                if theme::selectable_label(ui, self.tab == 3, &t.tab_base64) {
                    self.tab = 3;
                }
                if theme::selectable_label(ui, self.tab == 4, &t.tab_hash) {
                    self.tab = 4;
                }
                if theme::selectable_label(ui, self.tab == 5, &t.tab_checksum) {
                    self.tab = 5;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(&t.btn_about).clicked() {
                        self.show_about = !self.show_about;
                    }
                    egui::ComboBox::from_id_salt("lang_select")
                        .selected_text(&lang_names[current])
                        .show_ui(ui, |ui| {
                            for (idx, name) in lang_names.iter().enumerate() {
                                if theme::selectable_label(ui, current == idx, name) {
                                    current = idx;
                                }
                            }
                        });
                    ui.add_space(8.0);
                });
                ui.add_space(8.0);
            });
            ui.add_space(8.0);
        });

        // 语言切换时同步文案与窗口标题
        if current != prev_current {
            self.i18n.current = current;
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
        egui::CentralPanel::default().show(ui, |ui| match self.tab {
            0 => self.timestamp.ui(ui),
            1 => self.encoding.ui(ui),
            2 => self.url.ui(ui),
            3 => self.base64.ui(ui),
            4 => self.hash.ui(ui),
            _ => self.checksum.ui(ui),
        });

        // 关于窗口
        if self.show_about {
            self.about_window(ui.ctx(), &t);
        }
    }
}

impl ToolkitApp {
    /// 关于窗口: 版本/作者/联系方式等信息
    fn about_window(&mut self, ctx: &egui::Context, t: &Texts) {
        let (author_name, author_contact) = cargo_author();
        let mut open = self.show_about;
        let mut close_clicked = false;
        egui::Window::new(&t.about_title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut open)
            .show(ctx, |ui| {
                // 在 UI 代码最开始的位置设置一次
                ui.style_mut().interaction.selectable_labels = false;

                // 与标题栏保持足够间距
                ui.add_space(38.0);
                // 首行缩进两个字符, 表示段落开始
                ui.label(format!("　　{}", t.about_desc));
                ui.add_space(12.0);

                egui::Grid::new("about_grid")
                    .num_columns(2)
                    .spacing([16.0, 0.0])
                    .show(ui, |ui| {
                        
                        ui.label(format!("{}:", t.about_version));
                        ui.monospace(env!("CARGO_PKG_VERSION"));
                        ui.end_row();

                        ui.label(format!("{}:", t.about_author));
                        ui.label(author_name);
                        ui.end_row();

                        ui.label(format!("{}:", t.about_contact));
                        ui.label(author_contact);
                        ui.end_row();

                        ui.label(format!("{}:", t.about_license));
                        ui.label(env!("CARGO_PKG_LICENSE"));
                        ui.end_row();

                        ui.label(format!("{}:", t.about_built));
                        ui.monospace(env!("CARGO_PKG_VERSION"));
                        ui.end_row();
                    });

                ui.add_space(12.0);
                ui.vertical_centered(|ui| {
                    if ui.button(&t.btn_close).clicked() {
                        close_clicked = true;
                    }
                });
            });
        // 点击关闭按钮或窗口右上角 X 时关闭
        if close_clicked {
            open = false;
        }
        self.show_about = open;
    }
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
