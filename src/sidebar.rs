use quartz::{load_image, Canvas, GameObject, Image, ShapeType, Font, Color, Align, make_text_aligned};
use quartz::entropy::Entropy;
use image::{RgbaImage, Rgba};
use std::sync::{Arc, Mutex};
use crate::fish::{FishKind, GameAssets};

pub const GAME_WIDTH:    f32 = 3200.0;
pub const SIDEBAR_X:     f32 = 3200.0;
pub const SIDEBAR_W:     f32 = 640.0;
const TIMER_TOTAL:       f32 = 30.0;
const BORDER_SIZE:       f32 = 70.0; 
const CONTENT_X:         f32 = SIDEBAR_X + BORDER_SIZE;
const CONTENT_W:         f32 = SIDEBAR_W - BORDER_SIZE * 2.0;
const SIDEBAR_CENTER_X:  f32 = SIDEBAR_X + SIDEBAR_W / 2.0;

const TITLE_Y:       f32 = 80.0;
const TIMER_Y:       f32 = 195.0;
const SCORE_Y:       f32 = 310.0;
const DIVIDER_Y:     f32 = 415.0;
const CATCH_LABEL_Y: f32 = 440.0;
const ORDER_Y:       [f32; 3] = [530.0, 900.0, 1280.0];

const BUTTON_SIZE: f32 = 180.0;
const BUTTON_X:    f32 = SIDEBAR_X + SIDEBAR_W - BORDER_SIZE - BUTTON_SIZE;
const BUTTON_Y:    f32 = 2160.0 - BORDER_SIZE - BUTTON_SIZE;


#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FishSize { Small, Medium, Large }

impl FishSize {
    pub fn from_width(width: f32) -> Self {
        if width < 80.0       { FishSize::Small  }
        else if width < 100.0 { FishSize::Medium }
        else                  { FishSize::Large  }
    }

    fn display_width(self) -> f32 {
        match self { FishSize::Small => 160.0, FishSize::Medium => 210.0, FishSize::Large => 260.0 }
    }

    fn display_height(self) -> f32 { self.display_width() * 0.55 }

    fn label(self) -> &'static str {
        match self { FishSize::Small => "Small", FishSize::Medium => "Medium", FishSize::Large => "Large" }
    }
}

#[derive(Clone, Copy)]
pub struct OrderFish {
    pub kind: FishKind,
    pub size: FishSize,
}

impl OrderFish {
    fn label(self) -> String {
        let kind_name = match self.kind { FishKind::Gold => "Gold", FishKind::Blue => "Blue" };
        format!("{} {}", self.size.label(), kind_name)
    }
}

fn random_order(rng: &mut Entropy) -> OrderFish {
    let kind = if rng.range(0.0, 1.0) < 0.5 { FishKind::Gold } else { FishKind::Blue };
    let roll = rng.range(0.0, 3.0);
    let size = if roll < 1.0 { FishSize::Small } else if roll < 2.0 { FishSize::Medium } else { FishSize::Large };
    OrderFish { kind, size }
}

fn solid_rect(r: u8, g: u8, b: u8, a: u8, width: f32, height: f32) -> Image {
    let pixel = RgbaImage::from_pixel(1, 1, Rgba([r, g, b, a]));
    Image { shape: ShapeType::Rectangle(0.0, (width, height), 0.0), image: pixel.into(), color: None }
}

fn add_text_obj(canvas: &mut Canvas, name: &str, x: f32, y: f32, width: f32, height: f32, spec: quartz::TextSpec) {
    let mut obj = GameObject::build(name).size(width, height).position(x, y).finish();
    obj.set_text(spec);
    canvas.add_game_object(name.into(), obj);
}

fn mouse_is_over_button(mouse_pos: (f32, f32)) -> bool {
    mouse_pos.0 >= BUTTON_X && mouse_pos.0 <= BUTTON_X + BUTTON_SIZE
    && mouse_pos.1 >= BUTTON_Y && mouse_pos.1 <= BUTTON_Y + BUTTON_SIZE
}


#[derive(Clone)]
pub struct Sidebar {
    pub orders:    Vec<OrderFish>,
    pub score:     i32,
    pub timer:     f32,
    pub game_over: bool,
    settings_open: bool,
    button_hovered: bool,
    button_was_clicked: Arc<Mutex<bool>>,
    font:          Font,
    rng:           Entropy,
    orders_need_redraw: bool,
}

impl Sidebar {
    pub fn new(font: Font) -> Self {
        let mut rng = Entropy::from_seed(55_444_333_222);
        let orders = (0..3).map(|_| random_order(&mut rng)).collect();
        Self {
            orders, score: 0, timer: TIMER_TOTAL, game_over: false,
            settings_open: false, button_hovered: false,
            button_was_clicked: Arc::new(Mutex::new(false)),
            font, rng, orders_need_redraw: true,
        }
    }

    pub fn setup(&mut self, canvas: &mut Canvas, assets: &GameAssets) {
        canvas.add_game_object("sidebar_bg".into(),
            GameObject::build("sidebar_bg")
                .image(solid_rect(55, 33, 12, 255, SIDEBAR_W, 2160.0))
                .size(SIDEBAR_W, 2160.0)
                .position(SIDEBAR_X, 0.0)
                .finish());

        canvas.add_game_object("sidebar_frame".into(),
            GameObject::build("sidebar_frame")
                .image(load_image("assets/layers/bamboframe.png"))
                .size(SIDEBAR_W, 2160.0)
                .position(SIDEBAR_X, 0.0)
                .finish());

        canvas.add_game_object("settings_btn".into(),
            GameObject::build("settings_btn")
                .image(load_image("assets/layers/settings.png"))
                .size(BUTTON_SIZE, BUTTON_SIZE)
                .position(BUTTON_X, BUTTON_Y)
                .finish());

        self.draw_main_panel(canvas, assets);

        let flag = self.button_was_clicked.clone();
        canvas.on_mouse_press(move |_canvas, _button, mouse_pos| {
            if mouse_is_over_button(mouse_pos) {
                *flag.lock().unwrap() = true;
            }
        });
    }

    pub fn update(&mut self, canvas: &mut Canvas, assets: &GameAssets) {
        let button_clicked = {
            let mut flag = self.button_was_clicked.lock().unwrap();
            let was_clicked = *flag;
            *flag = false;
            was_clicked
        };

        if button_clicked {
            self.settings_open = !self.settings_open;
            self.switch_panel(canvas, assets);
        }

        if self.settings_open {
            let mouse_over = canvas.mouse_position()
                .map(mouse_is_over_button)
                .unwrap_or(false);

            if mouse_over != self.button_hovered {
                self.button_hovered = mouse_over;
                let color = if mouse_over { Color(255, 210, 60, 255) } else { Color(255, 255, 255, 255) };
                if let Some(obj) = canvas.get_game_object_mut("settings_btn") {
                    obj.set_text(make_text_aligned("Back", 40.0, &self.font, color, Align::Center));
                }
            }
            return;
        }

        if !self.game_over {
            self.timer = (self.timer - 0.016).max(0.0);
            if self.timer <= 0.0 { self.game_over = true; }
        }

        if let Some(obj) = canvas.get_game_object_mut("sidebar_timer") {
            obj.set_text(self.timer_text());
        }
        if let Some(obj) = canvas.get_game_object_mut("sidebar_score") {
            obj.set_text(self.score_text());
        }

        if self.orders_need_redraw {
            self.remove_order_objects(canvas);
            self.draw_orders(canvas, assets);
            self.orders_need_redraw = false;
        }
    }

    pub fn check_catch(&mut self, kind: FishKind, fish_width: f32) -> i32 {
        if self.game_over || self.orders.is_empty() { return 0; }

        let caught_size = FishSize::from_width(fish_width);
        let next_order = self.orders[0];
        let correct = next_order.kind == kind && next_order.size == caught_size;

        if correct {
            self.orders.remove(0);
            self.orders.push(random_order(&mut self.rng));
            self.score += 1;
        } else {
            self.score -= 1;
        }

        self.orders_need_redraw = true;
        if correct { 1 } else { -1 }
    }

    fn switch_panel(&mut self, canvas: &mut Canvas, assets: &GameAssets) {
        if self.settings_open {
            self.remove_main_panel(canvas);
            self.draw_settings_panel(canvas);
            if let Some(obj) = canvas.get_game_object_mut("settings_btn") {
                obj.set_text(make_text_aligned("Back", 40.0, &self.font, Color(255, 255, 255, 255), Align::Center));
            }
        } else {
            self.remove_settings_panel(canvas);
            self.draw_main_panel(canvas, assets);
            self.orders_need_redraw = true;
            if let Some(obj) = canvas.get_game_object_mut("settings_btn") {
                obj.set_image(load_image("assets/layers/settings.png"));
            }
        }
    }

    fn draw_main_panel(&mut self, canvas: &mut Canvas, assets: &GameAssets) {
        add_text_obj(canvas, "sidebar_title",
            CONTENT_X, TITLE_Y, CONTENT_W, 90.0,
            make_text_aligned("ORDERS", 80.0, &self.font, Color(255, 220, 80, 255), Align::Center));

        add_text_obj(canvas, "sidebar_timer",
            CONTENT_X, TIMER_Y, CONTENT_W, 110.0,
            self.timer_text());

        add_text_obj(canvas, "sidebar_score",
            CONTENT_X, SCORE_Y, CONTENT_W, 90.0,
            self.score_text());

        canvas.add_game_object("sidebar_line".into(),
            GameObject::build("sidebar_line")
                .image(solid_rect(180, 130, 50, 160, CONTENT_W, 4.0))
                .size(CONTENT_W, 4.0)
                .position(CONTENT_X, DIVIDER_Y)
                .finish());

        add_text_obj(canvas, "sidebar_catch",
            CONTENT_X, CATCH_LABEL_Y, CONTENT_W, 55.0,
            make_text_aligned("CATCH NEXT", 44.0, &self.font, Color(255, 190, 60, 255), Align::Center));

        self.draw_orders(canvas, assets);
    }

    fn remove_main_panel(&self, canvas: &mut Canvas) {
        for name in &["sidebar_title", "sidebar_timer", "sidebar_score", "sidebar_line", "sidebar_catch"] {
            canvas.remove_game_object(name);
        }
        self.remove_order_objects(canvas);
    }

    fn draw_settings_panel(&self, canvas: &mut Canvas) {
        add_text_obj(canvas, "settings_title",
            CONTENT_X, 80.0, CONTENT_W, 90.0,
            make_text_aligned("SETTINGS", 72.0, &self.font, Color(255, 220, 80, 255), Align::Center));

        let rows = [
            ("settings_row_0", "Line speed:  3"),
            ("settings_row_1", "Move speed:  6"),
            ("settings_row_2", "Max line:  1400"),
            ("settings_row_3", "Timer:  30s"),
            ("settings_row_4", "Max fish:  8"),
        ];

        for (index, (name, text)) in rows.iter().enumerate() {
            let y = 240.0 + index as f32 * 100.0;
            add_text_obj(canvas, name, CONTENT_X, y, CONTENT_W, 80.0,
                make_text_aligned(*text, 38.0, &self.font, Color(255, 255, 255, 255), Align::Center));
        }
    }

    fn remove_settings_panel(&self, canvas: &mut Canvas) {
        canvas.remove_game_object("settings_title");
        for i in 0..5 { canvas.remove_game_object(&format!("settings_row_{}", i)); }
    }

    fn draw_orders(&self, canvas: &mut Canvas, assets: &GameAssets) {
        for (index, order) in self.orders.iter().enumerate() {
            let fish_display_w = order.size.display_width();
            let fish_display_h = order.size.display_height();
            let fish_x = SIDEBAR_CENTER_X - fish_display_w / 2.0;
            let fish_y = ORDER_Y[index];

            let is_active = index == 0;
            let slot_color = if is_active { (110, 65, 18, 200) } else { (55, 30, 8, 140) };
            let slot_height = fish_display_h + 14.0 * 2.0 + 44.0;

            canvas.add_game_object(format!("order_slot_{}", index),
                GameObject::build(format!("order_slot_{}", index))
                    .image(solid_rect(slot_color.0, slot_color.1, slot_color.2, slot_color.3, CONTENT_W, slot_height))
                    .size(CONTENT_W, slot_height)
                    .position(CONTENT_X, fish_y - 14.0)
                    .finish());

            let fish_image = match order.kind {
                FishKind::Gold => assets.gold_right.clone(),
                FishKind::Blue => assets.blue_right.clone(),
            };
            canvas.add_game_object(format!("order_fish_{}", index),
                GameObject::build(format!("order_fish_{}", index))
                    .image(fish_image)
                    .size(fish_display_w, fish_display_h)
                    .position(fish_x, fish_y)
                    .finish());

            let label_color = if is_active { Color(255, 235, 80, 255) } else { Color(200, 175, 80, 220) };
            let label_size  = if is_active { 44.0 } else { 36.0 };
            add_text_obj(canvas, &format!("order_label_{}", index),
                CONTENT_X, fish_y + fish_display_h + 6.0, CONTENT_W, 70.0,
                make_text_aligned(&order.label(), label_size, &self.font, label_color, Align::Center));
        }
    }

    fn remove_order_objects(&self, canvas: &mut Canvas) {
        for i in 0..3 {
            canvas.remove_game_object(&format!("order_slot_{}", i));
            canvas.remove_game_object(&format!("order_fish_{}", i));
            canvas.remove_game_object(&format!("order_label_{}", i));
        }
    }

    fn timer_text(&self) -> quartz::TextSpec {
        let seconds = self.timer.ceil() as u32;
        let color = if self.timer > 10.0 { Color(80, 240, 80, 255) } else { Color(255, 60, 60, 255) };
        make_text_aligned(&format!("{}s", seconds), 72.0, &self.font, color, Align::Center)
    }

    fn score_text(&self) -> quartz::TextSpec {
        let color = if self.score >= 0 { Color(255, 230, 80, 255) } else { Color(255, 60, 60, 255) };
        make_text_aligned(&format!("Score: {}", self.score), 52.0, &self.font, color, Align::Center)
    }
}