use std::{
    path::Path,
    time::{Duration, Instant, SystemTime},
};

use eframe::{
    epaint::{
        ahash::{HashMap, HashSet},
        Shadow,
    },
    Frame,
};
use egui::{
    style::Margin, Align, Button, Color32, Context, ImageButton, Label, Layout, Order, Pos2, Rect,
    RichText, ScrollArea, Stroke, TextStyle, TextureHandle, TextureId, TextureOptions,
    TopBottomPanel, Vec2,
};

type LoadedImages = HashMap<String, (TextureHandle, Vec2)>;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    // #[serde(skip)]
    value: f32,

    // #[serde(skip)]
    recent_shots: Vec<RecentShot>,

    loaded_images: LoadedImages,

    theme: Theme,

    hovered_shots: HashSet<String>,

    icons: AppIcons,
}

struct AppIcons {
    video: egui_extras::RetainedImage,
    shot: egui_extras::RetainedImage,
    more: egui_extras::RetainedImage,
    gear: egui_extras::RetainedImage,
}

impl AppIcons {
    fn new() -> Self {
        Self {
            video: egui_extras::RetainedImage::from_svg_bytes_with_size(
                "video",
                include_bytes!("../assets/icons/video.svg"),
                egui_extras::image::FitTo::Original,
            )
            .unwrap(),
            shot: egui_extras::RetainedImage::from_svg_bytes_with_size(
                "shot",
                include_bytes!("../assets/icons/shot.svg"),
                egui_extras::image::FitTo::Original,
            )
            .unwrap(),
            more: egui_extras::RetainedImage::from_svg_bytes_with_size(
                "more",
                include_bytes!("../assets/icons/more.svg"),
                egui_extras::image::FitTo::Original,
            )
            .unwrap(),
            gear: egui_extras::RetainedImage::from_svg_bytes_with_size(
                "gear",
                include_bytes!("../assets/icons/gear.svg"),
                egui_extras::image::FitTo::Original,
            )
            .unwrap(),
        }
    }
}

struct Theme {
    brand_color: Color32,
}

impl Theme {
    // #ffc400
    fn new() -> Self {
        Self {
            // yellow color
            brand_color: Color32::from_rgb(0xff, 0xc4, 0)
                //.linear_multiply(0.3)
                .to_opaque(),
        }
    }
}

pub enum StorageKind {
    Local,
    Clipboard,
}

pub struct RecentShot {
    taken_at: Instant,
    name: String,
    stored_in: StorageKind,
    path_on_disk: String,
}

impl RecentShot {
    pub fn new(taken_at: Instant, name: String, stored_in: StorageKind) -> Self {
        Self {
            taken_at,
            // TODO temp hack
            path_on_disk: format!("./assets/recent_shots/{}", name),
            name,
            stored_in,
        }
    }
}

enum Message {
    Quit,
}

struct Messages {
    messages: Vec<Message>,
}

impl Messages {
    fn new() -> Self {
        Self { messages: vec![] }
    }

    fn add(&mut self, msg: Message) {
        self.messages.push(msg);
    }
}

// impl Default for TemplateApp {
//     fn default() -> Self {
//         Self {
//             // Example stuff:
//             label: "Hello World!".to_owned(),
//             value: 2.7,
//             recent_shots: Default::default(),
//         }
//     }
// }

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        let recent_shots = vec![
            RecentShot::new(
                Instant::now() - Duration::from_secs(60),
                "Screenshot 2023-01-31 at 10.16.00 PM.png".to_string(),
                StorageKind::Clipboard,
            ),
            RecentShot::new(
                Instant::now() - Duration::from_secs(6000),
                "Screenshot 2023-01-31 at 10.16.04 PM.png".to_string(),
                StorageKind::Local,
            ),
            RecentShot::new(
                Instant::now() - Duration::from_secs(360),
                "Screenshot 2023-01-31 at 10.16.11 PM.png".to_string(),
                StorageKind::Clipboard,
            ),
        ];

        Self {
            label: "CLOUDSHOT".to_string(),
            value: 0.32,
            loaded_images: HashMap::from_iter(recent_shots.iter().filter_map(|rs| {
                load_image_from_path(&Path::new(&rs.path_on_disk))
                    .ok()
                    .map(|img| {
                        let name = rs.name.clone();
                        let size = Vec2::new(img.size[0] as f32, img.size[1] as f32);
                        (
                            name,
                            (
                                cc.egui_ctx.load_texture(&rs.name, img, Default::default()),
                                size,
                            ),
                        )
                    })
            })),
            theme: Theme::new(),
            recent_shots,
            hovered_shots: Default::default(),
            icons: AppIcons::new(),
        }
    }
}

fn render_body(
    ctx: &Context,
    shots: &[RecentShot],
    hovered_shots: &mut HashSet<String>,
    images: &LoadedImages,
    theme: &Theme,
    icons: &AppIcons,
) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ScrollArea::vertical().show(ui, |ui| {
            for ((img, img_size), shot) in shots
                .iter()
                .filter_map(|s| images.get(&s.name).map(|img| (img, s)))
            {
                let is_hovered = hovered_shots.contains(&shot.name);

                let resp = egui::Frame::none()
                    // .fill(match is_hovered {
                    //     true => theme.brand_color.linear_multiply(1.).to_opaque(),
                    //     false => Color32::TRANSPARENT,
                    // })
                    .stroke(Stroke::new(
                        0.5,
                        match is_hovered {
                            true => theme.brand_color.linear_multiply(1.).to_opaque(),
                            false => ui.visuals().text_color().linear_multiply(0.3).to_opaque(),
                        },
                    ))
                    .inner_margin(Margin::same(5.0))
                    .rounding(5.)
                    //.shadow(Shadow::small_dark())
                    .show(ui, |ui| {
                        let aw = ui.available_width();
                        let ratio = 3. / 2.;
                        let card_size = Vec2::new(aw, aw / ratio);

                        let bounds = render_image_in_box(card_size, *img_size, img, ui);
                        // println!("## {:?}", bounds);

                        let info_rect = ui
                            .vertical(|ui| {
                                ui.label(&shot.name);
                                let storage_name = match shot.stored_in {
                                    StorageKind::Local => "Local",
                                    StorageKind::Clipboard => "Clipboard",
                                };
                                ui.label(
                                    RichText::new(format!(
                                        "{}, 9/15/2022 4:48 PM",
                                        storage_name,
                                        // SystemTime::now() // SystemTime:: (shot.taken_at)
                                    ))
                                    .text_style(TextStyle::Small)
                                    .color(ui.visuals().text_color().linear_multiply(0.4)),
                                );
                            })
                            .response
                            .rect;

                        if is_hovered {
                            let buttons_overlay = Rect::from_two_pos(
                                bounds.left_bottom() + Vec2::new(0., info_rect.height()),
                                bounds.right_bottom(),
                            );

                            let mut overlay_ui =
                                ui.child_ui(buttons_overlay, Layout::right_to_left(Align::Center));

                            overlay_ui.add(ImageButton::new(
                                icons.more.texture_id(ctx),
                                Vec2::new(12., 12.),
                            ));
                            overlay_ui.button("Copy Path");
                            overlay_ui.button("Open");
                        }
                    });

                if resp.response.hovered() {
                    hovered_shots.insert(shot.name.clone());
                } else {
                    hovered_shots.remove(&shot.name);
                }

                ui.add_space(10.);
            }
        });
    });
}

fn render_image_in_box(
    rect: Vec2,
    img_size: Vec2,
    img: &TextureHandle,
    // ctx: &Context,
    ui: &mut egui::Ui,
) -> Rect {
    let img_ratio = img_size.x / img_size.y;
    let box_ratio = rect.x / rect.y;

    let (w, h) = if img_ratio > box_ratio {
        (rect.x, rect.x / img_ratio)
    } else {
        (rect.y * img_ratio, rect.y)
    };

    let response = if img_ratio > box_ratio {
        // space below and above because image is wider
        ui.vertical(|ui| {
            ui.add_space((rect.y - h) / 2.0);
            ui.image(img, Vec2::new(w, h));
            ui.add_space((rect.y - h) / 2.0);
        })
    } else {
        // space on the left and on the right because image is taller
        ui.horizontal(|ui| {
            ui.add_space((rect.x - w) / 2.0);
            ui.image(img, Vec2::new(w, h));
            ui.add_space((rect.x - w) / 2.0);
        })
    };

    response.response.rect
}

fn load_image_from_path(path: &std::path::Path) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

fn render_footer(ctx: &Context, icons: &AppIcons) {
    TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.add_space(10.);
        egui::menu::bar(ui, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                ui.add_space(5.);

                let capture = ui.add(Button::image_and_text(
                    icons.shot.texture_id(ctx),
                    Vec2::new(18., 18.),
                    RichText::new("CAPTURE").text_style(egui::TextStyle::Heading),
                ));

                ui.add_space(10.);
                let record = ui.add(Button::image_and_text(
                    icons.video.texture_id(ctx),
                    Vec2::new(18., 18.),
                    RichText::new("RECORD").text_style(egui::TextStyle::Heading),
                ));
            });

            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    ui.add_space(5.);

                    let settings = ui.add(ImageButton::new(
                        icons.gear.texture_id(ctx),
                        Vec2::new(18., 18.),
                    ));
                }
            });
        });

        ui.add_space(10.);
    });
}

// define a TopBottomPanel widget
fn render_header_panel(ctx: &egui::Context, theme: &Theme, dispatch: &mut Messages) {
    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.add_space(10.);
        egui::menu::bar(ui, |ui| {
            // logo
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                ui.add_space(5.);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(0.3, 0.);
                    ui.label(RichText::new("CLOUD").text_style(egui::TextStyle::Heading));
                    ui.label(
                        RichText::new("SHOT")
                            .text_style(egui::TextStyle::Heading)
                            .color(theme.brand_color),
                    );
                });
            });

            // controls
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    ui.add_space(5.);

                    let close_btn = ui.add(Button::new(
                        RichText::new("❌").text_style(egui::TextStyle::Heading),
                    ));

                    if close_btn.clicked() {
                        dispatch.add(Message::Quit)
                    }
                }
            });
        });

        ui.add_space(10.);
    });
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            label,
            value,
            recent_shots,
            loaded_images,
            theme,
            hovered_shots,
            icons,
        } = self;

        let mut dispatch = Messages::new();

        render_header_panel(ctx, &theme, &mut dispatch);

        render_footer(ctx, &icons);

        render_body(
            ctx,
            recent_shots.as_slice(),
            hovered_shots,
            loaded_images,
            &theme,
            &icons,
        );

        for msg in dispatch.messages.drain(..) {
            match msg {
                Message::Quit => frame.close(),
            }
        }
    }
}
