use eframe::egui::{
    self, Button, Color32, CtxRef, FontDefinitions, FontFamily, Hyperlink, Label, Layout,
    Separator, TopBottomPanel, Window,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    sync::mpsc::{Receiver, SyncSender},
};

pub const PADDING: f32 = 5.0;
const WHITE: Color32 = Color32::from_rgb(255, 255, 255);
const BLACK: Color32 = Color32::from_rgb(0, 0, 0);
const CYAN: Color32 = Color32::from_rgb(0, 255, 255);
const RED: Color32 = Color32::from_rgb(255, 0, 0);

pub enum Msg {
    ApiKeySet(String),
}

#[derive(Serialize, Deserialize)]
pub struct HeadlinesConfig {
    pub dark_mode: bool,
    pub api_key: String,
}

impl Default for HeadlinesConfig {
    fn default() -> Self {
        Self {
            dark_mode: Default::default(),
            api_key: String::new(),
        }
    }
}

pub struct Headlines {
    pub articles: Vec<NewsCardData>,
    pub config: HeadlinesConfig,
    pub api_key_initialized: bool,
    pub news_rx: Option<Receiver<NewsCardData>>,
    pub app_tx: Option<SyncSender<Msg>>,
}

pub struct NewsCardData {
    pub title: String,
    pub desc: String,
    pub url: String,
}

impl Headlines {
    pub fn new() -> Headlines {
        let config: HeadlinesConfig =
            confy::load_path("/Users/lucas/.config/headlines/headlines.toml".to_string())
                .unwrap_or_default();

        Headlines {
            api_key_initialized: !config.api_key.is_empty(),
            articles: vec![],
            config,
            news_rx: None,
            app_tx: None,
        }
    }

    pub fn configure_fonts(&self, ctx: &CtxRef) {
        let mut font_def = FontDefinitions::default();
        font_def.font_data.insert(
            "Microsoft_YaHei".to_string(),
            Cow::Borrowed(include_bytes!("../../Microsoft_YaHei.ttf")),
        );
        font_def.family_and_size.insert(
            eframe::egui::TextStyle::Heading,
            (FontFamily::Proportional, 35.),
        );
        font_def.family_and_size.insert(
            eframe::egui::TextStyle::Body,
            (FontFamily::Proportional, 20.),
        );
        // 添加 Monospace 样式的配置
        font_def.family_and_size.insert(
            eframe::egui::TextStyle::Monospace,
            (FontFamily::Proportional, 20.),
        );
        font_def
            .fonts_for_family
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "Microsoft_YaHei".to_string());

        ctx.set_fonts(font_def);
    }

    pub fn render_news_cards(&self, ui: &mut eframe::egui::Ui) {
        for a in &self.articles {
            ui.add_space(PADDING);
            // 新闻标题
            let title = format!("▶ {}", a.title);
            if self.config.dark_mode {
                ui.colored_label(WHITE, title);
            } else {
                ui.colored_label(BLACK, title);
            }
            // 新闻描述
            ui.add_space(PADDING);
            let desc = Label::new(&a.desc).text_style(eframe::egui::TextStyle::Button);
            ui.add(desc);

            // 新闻相关超链接
            if self.config.dark_mode {
                ui.style_mut().visuals.hyperlink_color = CYAN;
            } else {
                ui.style_mut().visuals.hyperlink_color = RED;
            }
            ui.add_space(PADDING);
            ui.with_layout(Layout::right_to_left(), |ui| {
                ui.add(Hyperlink::new(&a.url).text("阅读更多 ⤴"));
            });
            ui.add_space(PADDING);
            ui.add(Separator::default());
        }
    }

    pub(crate) fn render_top_panel(&mut self, ctx: &CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        // 定义上下面板部件
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.);
            egui::menu::bar(ui, |ui| {
                // logo
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.add(Label::new("📓").text_style(egui::TextStyle::Heading));
                });
                // 控制按钮
                ui.with_layout(Layout::right_to_left(), |ui| {
                    let close_btn = ui.add(Button::new("❌").text_style(egui::TextStyle::Body));
                    if close_btn.clicked() {
                        frame.quit();
                    }
                    let _refresh_btn = ui.add(Button::new("🔄").text_style(egui::TextStyle::Body));
                    let theme_btn = ui.add(
                        Button::new({
                            if self.config.dark_mode {
                                "🌞"
                            } else {
                                "🌙"
                            }
                        })
                        .text_style(egui::TextStyle::Body),
                    );
                    if theme_btn.clicked() {
                        self.config.dark_mode = !self.config.dark_mode;
                    }
                });
            });
            ui.add_space(10.);
        });
    }

    pub fn preload_articles(&mut self) {
        if let Some(rx) = &self.news_rx {
            match rx.try_recv() {
                Ok(news_data) => {
                    self.articles.push(news_data);
                }
                Err(e) => {
                    tracing::warn!("Error receiving msg: {}", e);
                }
            }
        }
    }

    pub fn render_config(&mut self, ctx: &CtxRef) {
        Window::new("应用配置").show(ctx, |ui| {
            ui.label("在此输入你的 API_KEY (from newsapi.org)");
            let text_input = ui.text_edit_singleline(&mut self.config.api_key);
            if text_input.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                if let Err(e) = confy::store_path(
                    "/Users/lucas/.config/headlines/headlines.toml",
                    HeadlinesConfig {
                        dark_mode: self.config.dark_mode,
                        api_key: self.config.api_key.to_string(),
                    },
                ) {
                    tracing::error!("保存程序状态失败: {}", e);
                }

                self.api_key_initialized = true;
                if let Some(tx) = &self.app_tx {
                    let _ = tx.send(Msg::ApiKeySet(self.config.api_key.to_string()));
                }

                tracing::error!("api密钥已设置");
            }
            tracing::error!("{}", &self.config.api_key);
            // ui.label("If you havn't registered for the API_KEY, head over to");
            // ui.hyperlink("https://newsapi.org");
        });
    }
}
