//! HTTP 请求测试工具 (类 Postman)
//!
//! 基于 `ureq` 同步 HTTP 客户端 (rustls TLS), 纯 Rust 生态, 无系统 C 库依赖, 跨平台。
//! 请求在后台线程执行, 结果经 channel 送回, 界面线程仅做非阻塞轮询, 不卡 UI。
//!
//! 功能:
//! - 方法 (GET/POST/PUT/PATCH/DELETE/HEAD/OPTIONS) + URL
//! - 自定义请求头 (增删行, 发送前做合法性校验)
//! - 请求体: 无 / 表单 (x-www-form-urlencoded) / Form-Data (multipart/form-data,
//!   值以 `@` 开头表示文件) / 原文 (Content-Type 可选 text/json/xml/html)
//! - 响应: 状态码 / 耗时 / 大小 / 响应头 / 响应体
//!   (按 Content-Type charset 解码, JSON 可一键格式化)
//! - 会话内历史记录, 点击即可还原请求

use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use eframe::egui::{
    self, Color32, CursorIcon, FontFamily, FontId, Pos2, RichText, Sense, Ui, Vec2, vec2,
};
use encoding_rs::Encoding;

use crate::i18n::Texts;
use crate::theme;

/// HTTP 方法
const METHODS: [&str; 7] = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

/// 请求超时(整体: 连接 + 发送 + 读取)
const TIMEOUT: Duration = Duration::from_secs(30);

/// 响应体读取上限(防止超大响应撑爆内存)
const MAX_BODY: usize = 10 * 1024 * 1024;

/// 历史记录上限
const HISTORY_MAX: usize = 20;

/// 请求体模式
#[derive(Clone, Copy, PartialEq, Eq)]
enum BodyMode {
    None,
    /// application/x-www-form-urlencoded
    Form,
    /// multipart/form-data
    Multipart,
    /// 原文, Content-Type 由 RawType 决定
    Raw,
}

/// 原文请求体的内容类型
#[derive(Clone, Copy, PartialEq, Eq)]
enum RawType {
    Text,
    Json,
    Xml,
    Html,
}

impl RawType {
    /// 全部子类型(UI 展示顺序)
    const ALL: [RawType; 4] = [RawType::Text, RawType::Json, RawType::Xml, RawType::Html];

    /// 子类型显示名(JSON/XML/HTML 为通用技术名词, 不翻译)
    fn name(self) -> &'static str {
        return match self {
            RawType::Text => "Text",
            RawType::Json => "JSON",
            RawType::Xml => "XML",
            RawType::Html => "HTML",
        };
    }

    fn content_type(self) -> &'static str {
        return match self {
            RawType::Text => "text/plain; charset=utf-8",
            RawType::Json => "application/json; charset=utf-8",
            RawType::Xml => "application/xml; charset=utf-8",
            RawType::Html => "text/html; charset=utf-8",
        };
    }
}

impl HttpTool {
    /// 当前请求体模式下将发送的 Content-Type (None 模式返回 None)
    fn current_content_type(&self) -> Option<&str> {
        return match self.body_mode {
            BodyMode::None => None,
            BodyMode::Form => Some("application/x-www-form-urlencoded; charset=utf-8"),
            BodyMode::Multipart => Some("multipart/form-data; charset=utf-8; boundary=…"),
            BodyMode::Raw => Some(self.raw_type.content_type()),
        };
    }
}

/// 一次完整请求的参数(发送与历史还原共用)
#[derive(Clone, PartialEq)]
struct RequestSpec {
    /// 请求方法
    method: String,
    /// 请求 URL
    url: String,
    /// 请求头
    headers: Vec<(String, String)>,
    /// 请求体模式, none/form/multipart/raw,  raw 又分为 text/json/xml/html
    body_mode: BodyMode,
    /// 原文请求体的内容类型
    raw_type: RawType,
    /// 原文请求体
    body_raw: String,
    /// 表单数据(仅 Form 模式有)
    form: Vec<(String, String)>,
}

/// 一次响应的完整信息
struct HttpResponse {
    status: u16,
    /// 状态码文本
    status_text: String,
    /// 响应头
    headers: Vec<(String, String)>,
    /// 按 Content-Type charset 解码后的响应体
    body: String,
    /// JSON 格式化结果(响应体不是合法 JSON 时为 None)
    pretty: Option<String>,
    /// 实际读取的原始字节数
    size: usize,
    /// 响应体是否被截断
    truncated: bool,
    /// 解码警告(响应体不是 UTF-8 编码)
    warn_decode: bool,
    /// 请求耗时(请求 + 响应)
    elapsed: Duration,
}

/// HTTP 请求工具界面状态
pub struct HttpTool {
    t: Texts,

    // 请求参数
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body_mode: BodyMode,
    raw_type: RawType,
    body_raw: String,
    form: Vec<(String, String)>,

    // 响应
    loading: bool,
    result: Option<Result<HttpResponse, String>>,
    rx: Option<Receiver<Result<HttpResponse, String>>>,
    /// 响应体显示模式: 0 原文 / 1 JSON 格式化
    body_view: usize,
    /// 发送前置校验的警告(显示在请求卡片下方)
    warn: Option<String>,

    history: Vec<RequestSpec>,
}

impl HttpTool {
    pub fn new(t: Texts) -> Self {
        return Self {
            t,
            method: "GET".into(),
            url: String::new(),
            headers: vec![(String::new(), String::new())],
            body_mode: BodyMode::None,
            raw_type: RawType::Text,
            body_raw: String::new(),
            form: vec![(String::new(), String::new())],
            loading: false,
            result: None,
            rx: None,
            body_view: 0,
            warn: None,
            history: Vec::new(),
        };
    }

    /// 语言切换时更新文案
    pub fn set_lang(&mut self, t: Texts) {
        self.t = t;
    }

    /// 渲染工具界面
    pub fn ui(&mut self, ui: &mut Ui) {
        let t = self.t.clone();
        self.poll();

        let mut send = false;
        self.request_card(ui, &t, &mut send);
        theme::card_gap(ui);
        self.headers_card(ui, &t);
        theme::card_gap(ui);
        self.body_card(ui, &t);
        theme::card_gap(ui);
        self.response_card(ui, &t);

        let mut restore: Option<RequestSpec> = None;
        let mut clear = false;
        if !self.history.is_empty() {
            theme::card_gap(ui);
            theme::card(ui, |ui| {
                theme::output_header(ui, &t.http_history, "", "", |ui| {
                    if theme::button(ui, &t.http_clear).clicked() {
                        clear = true;
                    }
                });
                ui.spacing_mut().item_spacing.y = 2.0;
                for spec in &self.history {
                    if history_row(ui, &spec.method, &spec.url) {
                        restore = Some(spec.clone());
                    }
                }
            });
        }
        if let Some(spec) = restore {
            self.restore(&spec);
        }
        if clear {
            self.history.clear();
        }

        if send {
            self.try_send(&t);
        }
    }

    // -----------------------------------------------------------------------
    // 卡片
    // -----------------------------------------------------------------------

    /// 请求卡片: 方法 + URL + 发送
    fn request_card(&mut self, ui: &mut Ui, t: &Texts, send: &mut bool) {
        theme::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let mono = FontId::new(13.0, FontFamily::Monospace);
                egui::ComboBox::from_id_salt("http_method")
                    .selected_text(RichText::new(&self.method).font(mono.clone()))
                    .width(104.0)
                    .show_ui(ui, |ui| {
                        for m in METHODS {
                            if theme::menu_item(
                                ui,
                                self.method == m,
                                RichText::new(m).font(mono.clone()),
                            ) {
                                self.method = m.to_owned();
                            }
                        }
                    });
                // 发送按钮固定在行尾, URL 输入框占剩余宽度(避免溢出窗口)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    if theme::primary_button(ui, &t.http_send).clicked() {
                        *send = true;
                    }
                    let url_resp =
                        theme::text_input(ui, &mut self.url, &t.http_url_hint, f32::INFINITY);
                    if url_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        *send = true;
                    }
                });
            });
            if let Some(w) = &self.warn {
                theme::status(ui, theme::Level::Warn, w);
            }
        });
    }

    /// 请求头卡片: 键值对编辑
    fn headers_card(&mut self, ui: &mut Ui, t: &Texts) {
        theme::card(ui, |ui| {
            theme::field_label(ui, &t.http_headers);
            ui.add_space(2.0);
            if let Some(i) = kv_rows(
                ui,
                &mut self.headers,
                &t.http_header_key,
                &t.http_header_val,
                &t.http_remove,
            ) {
                self.headers.remove(i);
            }
            if theme::button(ui, &t.http_add).clicked() {
                self.headers.push((String::new(), String::new()));
            }
        });
    }

    /// 请求体卡片: 模式切换 + 对应编辑区
    fn body_card(&mut self, ui: &mut Ui, t: &Texts) {
        theme::card(ui, |ui| {
            // 可换行: 窄窗口下模式与子类型分段自动折行
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                theme::field_label(ui, &t.http_body);
                ui.add_space(2.0);
                let idx = match self.body_mode {
                    BodyMode::None => 0,
                    BodyMode::Form => 1,
                    BodyMode::Multipart => 2,
                    BodyMode::Raw => 3,
                };
                if let Some(i) = theme::segmented(
                    ui,
                    idx,
                    &[
                        &t.http_body_none,
                        &t.http_body_form,
                        &t.http_body_multipart,
                        &t.http_body_raw,
                    ],
                ) {
                    self.body_mode = match i {
                        0 => BodyMode::None,
                        1 => BodyMode::Form,
                        2 => BodyMode::Multipart,
                        _ => BodyMode::Raw,
                    };
                }
                // 原文模式下选择 Content-Type 子类型
                if self.body_mode == BodyMode::Raw {
                    ui.add_space(8.0);
                    let raw_idx = RawType::ALL
                        .iter()
                        .position(|&r| r == self.raw_type)
                        .unwrap_or(0);
                    let labels: Vec<&str> = RawType::ALL
                        .iter()
                        .map(|r| match r {
                            RawType::Text => t.http_raw_text.as_str(),
                            _ => r.name(),
                        })
                        .collect();
                    if let Some(i) = theme::segmented(ui, raw_idx, &labels) {
                        self.raw_type = RawType::ALL[i];
                    }
                }
            });
            // 展示当前模式实际发送的 Content-Type
            if let Some(ct) = self.current_content_type() {
                ui.horizontal(|ui| {
                    theme::hint_text(ui, "Content-Type");
                    ui.label(RichText::new(ct).font(FontId::new(12.5, FontFamily::Monospace)));
                });
                if self.body_mode == BodyMode::Multipart {
                    theme::hint_text(ui, &t.http_ct_boundary);
                }
            }
            match self.body_mode {
                BodyMode::None => {}
                BodyMode::Raw => {
                    theme::text_area(ui, &mut self.body_raw, &t.http_raw_hint, 4, true);
                }
                BodyMode::Form | BodyMode::Multipart => {
                    if let Some(i) = kv_rows(
                        ui,
                        &mut self.form,
                        &t.http_form_key,
                        &t.http_form_val,
                        &t.http_remove,
                    ) {
                        self.form.remove(i);
                    }
                    if theme::button(ui, &t.http_add).clicked() {
                        self.form.push((String::new(), String::new()));
                    }
                    if self.body_mode == BodyMode::Multipart {
                        theme::hint_text(ui, &t.http_multipart_hint);
                    }
                }
            }
        });
    }

    /// 响应卡片: 状态 / 响应头 / 响应体
    fn response_card(&mut self, ui: &mut Ui, t: &Texts) {
        let p = theme::pal(ui);
        let loading = self.loading;
        let mut body_view = self.body_view;
        theme::card(ui, |ui| {
            theme::field_label(ui, &t.http_response);
            if loading {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    ui.add(egui::Spinner::new().size(18.0));
                    ui.label(RichText::new(&t.http_sending).size(13.0).color(p.text_weak));
                });
                ui.ctx().request_repaint_after(Duration::from_millis(120));
            }

            match &self.result {
                Some(Ok(resp)) => {
                    // 状态行: 状态码 + 原因短语 + 耗时 + 大小
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        let color = status_color(p, resp.status);
                        ui.label(
                            RichText::new(format!("{} {}", resp.status, resp.status_text))
                                .font(FontId::new(14.0, FontFamily::Monospace))
                                .color(color),
                        );
                        ui.label(RichText::new("·").color(p.text_faint));
                        ui.label(
                            RichText::new(format_elapsed(resp.elapsed))
                                .font(FontId::new(13.0, FontFamily::Monospace))
                                .color(p.text_weak),
                        );
                        ui.label(RichText::new("·").color(p.text_faint));
                        ui.label(
                            RichText::new(format_bytes(resp.size))
                                .font(FontId::new(13.0, FontFamily::Monospace))
                                .color(p.text_weak),
                        );
                    });

                    // 响应头
                    if !resp.headers.is_empty() {
                        ui.add_space(4.0);
                        theme::field_label(
                            ui,
                            &format!("{} ({})", t.http_resp_headers, resp.headers.len()),
                        );
                        egui::ScrollArea::vertical()
                            .max_height(150.0)
                            .id_salt("http_resp_headers")
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                egui::Grid::new("http_resp_headers_grid")
                                    .num_columns(2)
                                    .spacing([18.0, 5.0])
                                    .show(ui, |ui| {
                                        for (k, v) in &resp.headers {
                                            ui.label(
                                                RichText::new(k)
                                                    .font(FontId::new(13.0, FontFamily::Monospace))
                                                    .color(p.text_weak),
                                            );
                                            ui.label(
                                                RichText::new(v)
                                                    .font(FontId::new(13.0, FontFamily::Monospace))
                                                    .color(p.text),
                                            );
                                            ui.end_row();
                                        }
                                    });
                            });
                    }

                    // 响应体
                    ui.add_space(8.0);
                    let shown: &str = if body_view == 1 {
                        resp.pretty.as_deref().unwrap_or(&resp.body)
                    } else {
                        &resp.body
                    };
                    theme::output_header(ui, &t.http_resp_body, &t.gen_copy, shown, |ui| {
                        if let Some(i) =
                            theme::segmented(ui, body_view, &[&t.http_body_raw, "JSON"])
                        {
                            body_view = i;
                        }
                    });
                    egui::ScrollArea::vertical()
                        .max_height(280.0)
                        .id_salt("http_resp_body")
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(shown)
                                    .font(FontId::new(13.0, FontFamily::Monospace))
                                    .color(p.text),
                            );
                        });
                    if resp.truncated {
                        theme::status(ui, theme::Level::Warn, &t.http_warn_truncated);
                    }
                    if resp.warn_decode {
                        theme::status(ui, theme::Level::Warn, &t.http_warn_decode);
                    }
                }
                Some(Err(e)) if !loading => {
                    theme::status(
                        ui,
                        theme::Level::Error,
                        &format!("{}: {e}", t.http_err_send),
                    );
                }
                None if !loading => theme::hint_text(ui, &t.http_resp_empty),
                _ => {}
            }
        });
        self.body_view = body_view;
    }

    // -----------------------------------------------------------------------
    // 发送 / 轮询 / 还原
    // -----------------------------------------------------------------------

    /// 收集当前请求参数(过滤空行)
    fn current_spec(&self) -> RequestSpec {
        return RequestSpec {
            method: self.method.clone(),
            url: self.url.trim().to_owned(),
            headers: self
                .headers
                .iter()
                .filter(|(k, _)| !k.is_empty())
                .cloned()
                .collect(),
            body_mode: self.body_mode,
            raw_type: self.raw_type,
            body_raw: self.body_raw.clone(),
            form: self
                .form
                .iter()
                .filter(|(k, _)| !k.is_empty())
                .cloned()
                .collect(),
        };
    }

    /// 校验并启动后台请求线程
    fn try_send(&mut self, t: &Texts) {
        let spec = self.current_spec();

        if !(spec.url.starts_with("http://") || spec.url.starts_with("https://")) {
            self.warn = Some(t.http_err_url.clone());
            return;
        }
        if spec
            .headers
            .iter()
            .any(|(k, v)| !is_header_name_valid(k) || !is_header_value_valid(v))
        {
            self.warn = Some(t.http_err_header.clone());
            return;
        }
        self.warn = None;

        // 历史记录(新的在前, 限量)
        self.history.insert(0, spec.clone());
        if self.history.len() > HISTORY_MAX {
            self.history.truncate(HISTORY_MAX);
        }

        let (tx, rx) = mpsc::channel();
        let spec_for_thread = spec;
        let spawned = thread::Builder::new()
            .name("http-request".into())
            .spawn(move || {
                let result = perform(&spec_for_thread);
                let _ = tx.send(result);
            });
        if spawned.is_err() {
            self.result = Some(Err("failed to spawn request thread".into()));
            self.loading = false;
            return;
        }
        self.rx = Some(rx);
        self.loading = true;
    }

    /// 非阻塞轮询后台线程结果
    fn poll(&mut self) {
        let Some(rx) = &self.rx else {
            return;
        };
        match rx.try_recv() {
            Ok(result) => {
                self.rx = None;
                self.loading = false;
                self.result = Some(result);
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                // 线程异常结束且未发送结果, 保持旧结果展示
                self.rx = None;
                self.loading = false;
            }
        }
    }

    /// 从历史记录还原请求参数
    fn restore(&mut self, spec: &RequestSpec) {
        self.method = spec.method.clone();
        self.url = spec.url.clone();
        self.headers = if spec.headers.is_empty() {
            vec![(String::new(), String::new())]
        } else {
            spec.headers.clone()
        };
        self.body_mode = spec.body_mode;
        self.raw_type = spec.raw_type;
        self.body_raw = spec.body_raw.clone();
        self.form = if spec.form.is_empty() && spec.body_mode == BodyMode::Form {
            vec![(String::new(), String::new())]
        } else {
            spec.form.clone()
        };
        self.body_view = 0;
        self.warn = None;
        self.rx = None;
        self.loading = false;
        self.result = None;
    }
}

// ---------------------------------------------------------------------------
// 请求执行(后台线程)
// ---------------------------------------------------------------------------

/// 执行请求并读取完整响应(阻塞, 仅供后台线程调用)
fn perform(spec: &RequestSpec) -> Result<HttpResponse, String> {
    let start = Instant::now();
    let mut req = ureq::request(&spec.method, &spec.url).timeout(TIMEOUT);
    for (k, v) in &spec.headers {
        req = req.set(k, v);
    }

    let result = match spec.body_mode {
        BodyMode::None => req.call(),
        BodyMode::Form => req
            .set(
                "Content-Type",
                "application/x-www-form-urlencoded; charset=utf-8",
            )
            .send_string(&form_encode(&spec.form)),
        BodyMode::Multipart => match build_multipart(&spec.form) {
            Ok((boundary, bytes)) => req
                .set(
                    "Content-Type",
                    &format!("multipart/form-data; charset=utf-8; boundary={boundary}"),
                )
                .send_bytes(&bytes),
            Err(e) => return Err(e),
        },
        BodyMode::Raw => req
            .set("Content-Type", spec.raw_type.content_type())
            .send_string(&spec.body_raw),
    };

    let resp = match result {
        Ok(r) => r,
        // 4xx/5xx 同样是有效响应, 转为正常返回以便展示其内容
        Err(ureq::Error::Status(_, r)) => r,
        Err(e) => return Err(e.to_string()),
    };
    return Ok(read_response(resp, start.elapsed()));
}

/// 读取响应的全部内容并解码为文本
fn read_response(resp: ureq::Response, elapsed: Duration) -> HttpResponse {
    let status = resp.status();
    let status_text = resp.status_text().to_owned();
    let headers: Vec<(String, String)> = resp
        .headers_names()
        .into_iter()
        .map(|name| {
            let val = resp.header(&name).unwrap_or("").to_owned();
            return (name, val);
        })
        .collect();
    // ureq 的 content_type() 不含 charset 参数, 需从完整头中取
    let content_type = resp.header("content-type").unwrap_or("").to_owned();
    let enc = charset_of(&content_type);

    let mut buf: Vec<u8> = Vec::new();
    let _ = resp
        .into_reader()
        .take((MAX_BODY + 1) as u64)
        .read_to_end(&mut buf);
    let truncated = if buf.len() > MAX_BODY {
        buf.truncate(MAX_BODY);
        true
    } else {
        false
    };

    let (body, _, had_errors) = enc.decode(&buf);
    let body = body.into_owned();
    let pretty = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|v| serde_json::to_string_pretty(&v).ok());

    return HttpResponse {
        status,
        status_text,
        headers,
        body,
        pretty,
        size: buf.len(),
        truncated,
        warn_decode: had_errors,
        elapsed,
    };
}

// ---------------------------------------------------------------------------
// 辅助函数
// ---------------------------------------------------------------------------

/// 键值对编辑器(每行: 名称输入 + 值输入 + 删除按钮), 返回被删除的行号
fn kv_rows(
    ui: &mut Ui,
    rows: &mut [(String, String)],
    key_hint: &str,
    val_hint: &str,
    remove_tip: &str,
) -> Option<usize> {
    let mut deleted = None;
    for (i, (k, v)) in rows.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            theme::text_input(ui, k, key_hint, 150.0);
            // 删除按钮固定在行尾, 值输入框占剩余宽度(避免溢出窗口)
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                if remove_button(ui, remove_tip) {
                    deleted = Some(i);
                }
                theme::text_input(ui, v, val_hint, f32::INFINITY);
            });
        });
    }
    return deleted;
}

/// 行删除小按钮: 悬停变红的 ✕
fn remove_button(ui: &mut Ui, tooltip: &str) -> bool {
    let p = theme::pal(ui);
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let hover = ui
            .ctx()
            .animate_bool_with_time(resp.id, resp.hovered(), theme::ANIM);
        let color = p.text_weak.lerp_to_gamma(p.error, hover);
        let c = rect.center();
        let stroke = eframe::egui::Stroke::new(1.5, color);
        ui.painter()
            .line_segment([c - vec2(4.0, 4.0), c + vec2(4.0, 4.0)], stroke);
        ui.painter()
            .line_segment([c - vec2(-4.0, 4.0), c + vec2(-4.0, 4.0)], stroke);
    }
    return resp
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(tooltip)
        .clicked();
}

/// 历史记录行: 方法徽标 + URL, 点击返回 true
fn history_row(ui: &mut Ui, method: &str, url: &str) -> bool {
    let p = theme::pal(ui);
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let hover = ui
            .ctx()
            .animate_bool_with_time(resp.id, resp.hovered(), theme::ANIM);
        let painter = ui.painter();
        if hover > 0.0 {
            painter.rect_filled(
                rect,
                eframe::egui::CornerRadius::same(6),
                p.raised.gamma_multiply(hover),
            );
        }
        let m_galley = theme::layout(
            ui,
            RichText::new(method)
                .font(FontId::new(12.0, FontFamily::Monospace))
                .strong(),
            48.0,
        );
        painter.galley(
            Pos2::new(rect.left() + 8.0, rect.center().y - m_galley.size().y / 2.0),
            m_galley,
            method_color(p, method),
        );
        let u_galley = theme::layout(ui, RichText::new(url).size(13.0), rect.width() - 70.0);
        painter.galley(
            Pos2::new(
                rect.left() + 62.0,
                rect.center().y - u_galley.size().y / 2.0,
            ),
            u_galley,
            p.text_weak.lerp_to_gamma(p.text, hover),
        );
    }
    return resp.on_hover_cursor(CursorIcon::PointingHand).clicked();
}

/// 方法徽标颜色
fn method_color(p: &theme::Palette, method: &str) -> Color32 {
    return match method {
        "GET" => p.success,
        "POST" => p.accent,
        "PUT" | "PATCH" => p.warn,
        "DELETE" => p.error,
        _ => p.text_weak,
    };
}

/// 状态码颜色: 2xx 成功 / 3xx 重定向 / 其余错误
fn status_color(p: &theme::Palette, code: u16) -> Color32 {
    if (200..300).contains(&code) {
        return p.success;
    }
    if (300..400).contains(&code) {
        return p.warn;
    }
    return p.error;
}

/// 从 Content-Type 头解析字符集(忽略大小写, 默认 UTF-8)
fn charset_of(content_type: &str) -> &'static Encoding {
    let charset = content_type
        .split(';')
        .find_map(|part| {
            let (k, v) = part.trim().split_once('=')?;
            if !k.eq_ignore_ascii_case("charset") {
                return None;
            }
            return Some(v.trim().trim_matches('"'));
        })
        .unwrap_or("");
    return Encoding::for_label(charset.as_bytes()).unwrap_or(encoding_rs::UTF_8);
}

/// 表单键值对编码为 application/x-www-form-urlencoded (空格→+, 保留 A-Za-z0-9-_.*)
fn form_encode(pairs: &[(String, String)]) -> String {
    let parts: Vec<String> = pairs
        .iter()
        .filter(|(k, _)| !k.is_empty())
        .map(|(k, v)| format!("{}={}", form_url_encode(k), form_url_encode(v)))
        .collect();
    return parts.join("&");
}

/// 单个值的表单编码
fn form_url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'*' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    return out;
}

/// multipart 请求序号(参与 boundary 生成, 保证同一进程内不重复)
static MULTIPART_SEQ: AtomicU32 = AtomicU32::new(0);

/// 生成 multipart boundary
fn make_boundary() -> String {
    let seq = MULTIPART_SEQ.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    return format!("----devtoolkit-{nanos:08x}-{seq:04x}");
}

/// 构建multipart/form-data 请求体
///
/// 键值对的值以 `@` 开头时表示文件路径(curl 惯例), 读取文件内容作为文件字段发送。
/// 返回 (boundary, 请求体字节)。
fn build_multipart(pairs: &[(String, String)]) -> Result<(String, Vec<u8>), String> {
    let boundary = make_boundary();
    let mut body: Vec<u8> = Vec::new();
    for (name, value) in pairs {
        if name.is_empty() {
            continue;
        }
        if let Some(path) = value.strip_prefix('@') {
            // 文件字段
            let path = path.trim();
            let data = std::fs::read(path).map_err(|e| format!("read file `{path}`: {e}"))?;
            let filename = Path::new(path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("file");
            body.extend_from_slice(
                format!(
                    "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
                )
                .as_bytes(),
            );
            body.extend_from_slice(&data);
            body.extend_from_slice(b"\r\n");
        } else {
            // 文本字段
            body.extend_from_slice(
                format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n")
                    .as_bytes(),
            );
        }
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    return Ok((boundary, body));
}

/// 请求头名称合法性(RFC 7230 token 字符)
fn is_header_name_valid(name: &str) -> bool {
    return !name.is_empty()
        && name
            .bytes()
            .all(|b| matches!(b, b'!' | b'#'..=b'\'' | b'*' | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z'));
}

/// 请求头值合法性(禁止 CR/LF/控制字符, 允许 TAB 与高位字节)
fn is_header_value_valid(value: &str) -> bool {
    return value
        .bytes()
        .all(|b| b == b'\t' || (0x20..0x7f).contains(&b) || b >= 0x80);
}

/// 字节数格式化
fn format_bytes(n: usize) -> String {
    if n < 1024 {
        return format!("{n} B");
    }
    if n < 1024 * 1024 {
        return format!("{:.1} KB", n as f64 / 1024.0);
    }
    return format!("{:.2} MB", n as f64 / (1024.0 * 1024.0));
}

/// 耗时格式化
fn format_elapsed(d: Duration) -> String {
    let ms = d.as_millis();
    if ms < 1000 {
        return format!("{ms} ms");
    }
    return format!("{:.2} s", d.as_secs_f64());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_encoding() {
        assert_eq!(form_encode(&[("a".into(), "1".into())]), "a=1");
        assert_eq!(
            form_encode(&[("k 1".into(), "值 v".into()), ("".into(), "跳过".into())]),
            "k+1=%E5%80%BC+v"
        );
        assert_eq!(form_url_encode("a&b=c"), "a%26b%3Dc");
        assert_eq!(form_url_encode("a-b.c*d_e"), "a-b.c*d_e");
    }

    #[test]
    fn header_validation() {
        assert!(is_header_name_valid("X-Token"));
        assert!(is_header_name_valid("x_custom.header~"));
        assert!(!is_header_name_valid(""));
        assert!(!is_header_name_valid("X Token"));
        assert!(!is_header_name_valid("X:Token"));
        assert!(is_header_value_valid("plain value"));
        assert!(is_header_value_valid("tab\tvalue"));
        assert!(!is_header_value_valid("injection\r\nX-Evil: 1"));
        assert!(!is_header_value_valid("nul\0"));
    }

    #[test]
    fn charset_parsing() {
        assert_eq!(charset_of("application/json"), encoding_rs::UTF_8);
        assert_eq!(charset_of("text/html; charset=GBK"), encoding_rs::GBK);
        assert_eq!(
            charset_of("Text/HTML; CHARSET=\"utf-8\""),
            encoding_rs::UTF_8
        );
        assert_eq!(
            charset_of("text/plain; charset=unknown-x"),
            encoding_rs::UTF_8
        );
    }

    #[test]
    fn formatting() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2.0 KB");
        assert_eq!(format_bytes(3 * 1024 * 1024), "3.00 MB");
        assert_eq!(format_elapsed(Duration::from_millis(145)), "145 ms");
        assert_eq!(format_elapsed(Duration::from_millis(1500)), "1.50 s");
    }

    #[test]
    fn raw_content_types() {
        assert_eq!(RawType::Text.content_type(), "text/plain; charset=utf-8");
        assert_eq!(
            RawType::Json.content_type(),
            "application/json; charset=utf-8"
        );
        assert_eq!(
            RawType::Xml.content_type(),
            "application/xml; charset=utf-8"
        );
        assert_eq!(RawType::Html.content_type(), "text/html; charset=utf-8");
    }

    #[test]
    fn multipart_text_fields() {
        let (boundary, body) =
            build_multipart(&[("a".into(), "1".into()), ("".into(), "跳过".into())]).unwrap();
        assert!(boundary.starts_with("----devtoolkit-"));
        let text = String::from_utf8(body).unwrap();
        assert_eq!(
            text,
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"a\"\r\n\r\n1\r\n--{boundary}--\r\n"
            )
        );
    }

    #[test]
    fn multipart_file_field() {
        // 临时文件(含非 UTF-8 二进制内容)验证文件字段
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "devtoolkit-multipart-test-{}.bin",
            std::process::id()
        ));
        std::fs::write(&path, [0xFF, 0x00, 0xAB]).unwrap();
        let file_path = path.to_string_lossy().into_owned();

        let (boundary, body) =
            build_multipart(&[("file".into(), format!("@{file_path}"))]).unwrap();
        let _ = std::fs::remove_file(&path);
        let text = String::from_utf8_lossy(&body);
        let filename = Path::new(&file_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let header = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
        );
        assert!(text.starts_with(&header));
        assert!(text.ends_with(&format!("\r\n--{boundary}--\r\n")));
        // 二进制内容原样保留
        let expected_len = header.len() + 3 + format!("\r\n--{boundary}--\r\n").len();
        assert_eq!(body.len(), expected_len);
    }

    #[test]
    fn multipart_missing_file_is_error() {
        let result = build_multipart(&[("f".into(), "@Z:/no/such/file.bin".into())]);
        assert!(result.is_err());
    }
}
