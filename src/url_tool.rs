//! URL 编解码工具
//!
//! 支持两种模式:
//! - 标准(表单式): 空格→`+`, 保留 `A-Za-z0-9-_.*`(application/x-www-form-urlencoded)
//! - 安全(RFC 3986): 空格→`%20`, 保留 `A-Za-z0-9-_.~!*'()`
//!
//! 解码时对无效的 `%` 转义序列按字面保留, 并给出警告提示。

use eframe::egui;

use crate::i18n::Texts;
use crate::theme;

/// 编码模式
#[derive(Clone, Copy, PartialEq, Eq)]
enum UrlMode {
    /// 标准(表单式)
    Std,
    /// 安全(RFC 3986)
    Safe,
}

/// 最近一次执行的操作(输入变化或模式切换时自动重放)
#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    Encode,
    Decode,
}

/// URL 编解码工具界面状态
pub struct UrlTool {
    t: Texts,

    mode: UrlMode,
    action: Action,
    input: String,
    output: String,
    /// 解码时是否遇到无效的 % 序列
    warn: bool,
    last_key: (UrlMode, Action, String),
}

impl UrlTool {
    pub fn new(t: Texts) -> Self {
        Self {
            t,
            mode: UrlMode::Std,
            action: Action::Encode,
            input: String::new(),
            output: String::new(),
            warn: false,
            last_key: (UrlMode::Std, Action::Encode, String::new()),
        }
    }

    /// 语言切换时更新文案
    pub fn set_lang(&mut self, t: Texts) {
        self.t = t;
    }

    /// 渲染工具界面
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let t = self.t.clone();
        self.update();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(&t.url_title).size(20.0).strong());
            ui.add_space(8.0);

            theme::card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(&t.url_mode);
                    if theme::selectable_label(ui, self.mode == UrlMode::Std, &t.url_mode_std) {
                        self.mode = UrlMode::Std;
                    }
                    if theme::selectable_label(ui, self.mode == UrlMode::Safe, &t.url_mode_safe) {
                        self.mode = UrlMode::Safe;
                    }
                });
            });
            ui.add_space(12.0);

            theme::card(ui, |ui| {
                ui.label(&t.gen_input);
                ui.add(
                    egui::TextEdit::singleline(&mut self.input)
                        .hint_text(t.url_hint.clone())
                        .margin(egui::Margin::symmetric(8, 14))
                        .desired_width(f32::INFINITY)
                        .vertical_align(egui::Align::Center),
                );

                ui.horizontal(|ui| {
                    if ui.button(&t.gen_encode).clicked() {
                        self.set_action(Action::Encode);
                    }
                    if ui.button(&t.gen_decode).clicked() {
                        self.set_action(Action::Decode);
                    }
                });

                if self.warn {
                    ui.colored_label(theme::WARN, &t.url_warn_invalid);
                }

                ui.label(&t.gen_output);
                ui.add(
                    egui::TextEdit::multiline(&mut self.output)
                        .desired_rows(4)
                        .desired_width(f32::INFINITY),
                );

                ui.horizontal(|ui| {
                    if !self.output.is_empty() {
                        theme::copy_button(ui, &t.gen_copy, &self.output);
                    }
                });
            });
            ui.add_space(8.0);
        });
    }

    /// 切换操作(编码/解码)并立即重算
    fn set_action(&mut self, action: Action) {
        self.action = action;
        // 强制重算
        self.last_key = (self.mode, self.action, String::new());
        self.update();
    }

    /// 输入/模式变化时重放最近一次操作
    fn update(&mut self) {
        let key = (self.mode, self.action, self.input.clone());
        if self.last_key == key {
            return;
        }
        self.last_key = key;

        match self.action {
            Action::Encode => {
                self.output = encode(&self.input, self.mode);
                self.warn = false;
            }
            Action::Decode => {
                let (text, warn) = decode(&self.input, self.mode);
                self.output = text;
                self.warn = warn;
            }
        }
    }
}

/// 按指定模式对文本进行 URL 编码
fn encode(input: &str, mode: UrlMode) -> String {
    let mut out = String::new();
    for &b in input.as_bytes() {
        match (b, mode) {
            // 字母数字总是保留
            (b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9', _) => out.push(b as char),
            // 标准模式保留 - _ . *
            (b'-' | b'_' | b'.' | b'*', UrlMode::Std) => out.push(b as char),
            // 安全模式额外保留 - _ . ~ ! * ' ( )
            (b'-' | b'_' | b'.' | b'~' | b'!' | b'*' | b'\'' | b'(' | b')', UrlMode::Safe) => {
                out.push(b as char)
            }
            (b' ', UrlMode::Std) => out.push('+'),
            (b' ', UrlMode::Safe) => out.push_str("%20"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// 按指定模式解码 URL 编码的文本
///
/// 返回 (解码结果, 是否包含无效的 % 序列)
fn decode(input: &str, mode: UrlMode) -> (String, bool) {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut warn = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                if let (Some(h), Some(l)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                    out.push(h * 16 + l);
                    i += 3;
                    continue;
                }
                // 无效转义: 按字面保留
                warn = true;
                out.push(b'%');
                i += 1;
            }
            // 标准模式将 + 视为空格
            b'+' if mode == UrlMode::Std => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    (String::from_utf8_lossy(&out).into_owned(), warn)
}

/// 十六进制字符对应的数值
fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
