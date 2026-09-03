//! Base64 编解码工具
//!
//! 支持标准字符表(`+/`)与 URL 安全字符表(`-_`), 解码时自动忽略空白字符,
//! 对非法的 Base64 输入给出错误提示。

use eframe::egui;

use crate::i18n::Texts;
use crate::theme;

/// 字符表
#[derive(Clone, Copy, PartialEq, Eq)]
enum Alphabet {
    /// 标准: `A-Za-z0-9+/`
    Std,
    /// URL 安全: `A-Za-z0-9-_`
    UrlSafe,
}

/// 最近一次执行的操作(输入变化或字符表切换时自动重放)
#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    Encode,
    Decode,
}

/// Base64 编解码工具界面状态
pub struct Base64Tool {
    t: Texts,

    alphabet: Alphabet,
    action: Action,
    input: String,
    output: String,
    error: bool,
    last_key: (Alphabet, Action, String),
}

impl Base64Tool {
    pub fn new(t: Texts) -> Self {
        Self {
            t,
            alphabet: Alphabet::Std,
            action: Action::Encode,
            input: String::new(),
            output: String::new(),
            error: false,
            last_key: (Alphabet::Std, Action::Encode, String::new()),
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

        theme::card(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                theme::field_label(ui, &t.b64_alphabet);
                ui.add_space(4.0);
                let alphabet = if self.alphabet == Alphabet::Std { 0 } else { 1 };
                if let Some(i) = theme::segmented(ui, alphabet, &[&t.b64_std, &t.b64_urlsafe]) {
                    self.alphabet = if i == 0 { Alphabet::Std } else { Alphabet::UrlSafe };
                }
                ui.add_space(16.0);
                let action = if self.action == Action::Encode { 0 } else { 1 };
                if let Some(i) = theme::segmented(ui, action, &[&t.gen_encode, &t.gen_decode]) {
                    self.set_action(if i == 0 { Action::Encode } else { Action::Decode });
                }
            });
        });
        theme::card_gap(ui);

        theme::card(ui, |ui| {
            theme::field_label(ui, &t.gen_input);
            theme::text_area(ui, &mut self.input, &t.b64_hint, 3, false);

            if self.error {
                theme::status(ui, theme::Level::Error, &t.b64_err_invalid);
            }

            ui.add_space(6.0);
            theme::output_header(ui, &t.gen_output, &t.gen_copy, &self.output, |_| {});
            theme::text_area(ui, &mut self.output, "", 4, true);
        });
    }

    /// 切换操作(编码/解码)并立即重算
    fn set_action(&mut self, action: Action) {
        self.action = action;
        // 强制重算
        self.last_key = (self.alphabet, self.action, String::new());
        self.update();
    }

    /// 输入/字符表变化时重放最近一次操作
    fn update(&mut self) {
        let key = (self.alphabet, self.action, self.input.clone());
        if self.last_key == key {
            return;
        }
        self.last_key = key;

        match self.action {
            Action::Encode => {
                self.output = encode(&self.input, self.alphabet);
                self.error = false;
            }
            Action::Decode => match decode(&self.input, self.alphabet) {
                Ok(text) => {
                    self.output = text;
                    self.error = false;
                }
                Err(()) => {
                    self.output.clear();
                    self.error = true;
                }
            },
        }
    }
}

/// 标准 Base64 字符表
const B64_STD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
/// URL 安全 Base64 字符表
const B64_URL_SAFE: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// 将文本编码为 Base64
fn encode(text: &str, alphabet: Alphabet) -> String {
    let table = match alphabet {
        Alphabet::Std => B64_STD,
        Alphabet::UrlSafe => B64_URL_SAFE,
    };
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        out.push(table[(b0 >> 2) as usize] as char);
        out.push(table[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(if chunk.len() > 1 {
            table[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            table[(b2 & 0x3F) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// 将 Base64 解码为文本, 非法输入返回 Err
fn decode(input: &str, alphabet: Alphabet) -> Result<String, ()> {
    let table = match alphabet {
        Alphabet::Std => B64_STD,
        Alphabet::UrlSafe => B64_URL_SAFE,
    };
    // 字符 -> 值(非法为 -1)
    let mut values = [-1i16; 128];
    for (i, &c) in table.iter().enumerate() {
        values[c as usize] = i as i16;
    }

    let mut bytes = Vec::with_capacity(input.len() / 4 * 3);
    let mut buf: u32 = 0;
    let mut nbits = 0;

    for c in input.chars() {
        if c.is_whitespace() {
            continue; // 忽略空白
        }
        if c == '=' {
            break; // 填充结束
        }
        let cu = c as u32;
        if cu >= 128 {
            return Err(());
        }
        let v = values[cu as usize];
        if v < 0 {
            return Err(());
        }
        buf = (buf << 6) | v as u32;
        nbits += 6;
        if nbits >= 8 {
            nbits -= 8;
            bytes.push((buf >> nbits) as u8);
        }
    }

    // 剩余 6 bit 说明字符数 mod 4 == 1, 非法
    if nbits == 6 {
        return Err(());
    }
    String::from_utf8(bytes).map_err(|_| ())
}
