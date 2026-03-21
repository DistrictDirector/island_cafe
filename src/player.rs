use flowmango::prelude::*;
use quartz::{GameObject, Image, ShapeType};
use image::{RgbaImage, Rgba};
use crate::fish::GameAssets;

const GAME_WIDTH: f32 = 3200.0;

// Creates a thin brown rectangle used as the fishing line
fn line_image(h: f32) -> Image {
    let pixel = RgbaImage::from_pixel(1, 1, Rgba([101, 67, 33, 255]));
    Image { shape: ShapeType::Rectangle(0.0, (4.0, h), 0.0), image: pixel.into(), color: None }
}

#[derive(Clone)]
pub struct Player {
    pub facing_left: bool,
    pub line_len: f32,
    pub player_w: f32,
    pub player_h: f32,
    pub max_line: f32,
    pub hook_w: f32,
    pub hook_h: f32,
    last_line: f32,
    rowing: Option<SoundHandle>,
    reeling: Option<SoundHandle>,
    move_speed: f32,
    line_speed: f32,
    rod_tip_right: (f32, f32),
    rod_tip_left: (f32, f32),
}

impl Player {
    pub fn new(player_w: f32, player_h: f32, hook_w: f32, hook_h: f32) -> Self {
        Self {
            facing_left: false,
            line_len: 0.0,
            last_line: -1.0,
            rowing: None,
            reeling: None,
            player_w,
            player_h,
            move_speed: 6.0,
            max_line: 1400.0,
            line_speed: 3.0,
            hook_w,
            hook_h,
            // Where the rod tip is relative to the player sprite (normalised 0-1)
            rod_tip_right: (0.85, 0.20),
            rod_tip_left: (0.15, 0.20),
        }
    }

    pub fn stop_rowing(&mut self) {
        if let Some(handle) = self.rowing.take() {
            handle.fade_out(0.25);
        }
    }

    pub fn handle_movement(&mut self, canvas: &mut Canvas, assets: &GameAssets, left: bool, right: bool) {
        // Can't move while the line is in the water
        if self.line_len > 0.0 { return; }

        let moving = left || right;

        if moving {
            let done = self.rowing.as_ref().map_or(true, |h| h.is_finished());
            if done {
                self.rowing = Some(canvas.play_sound_with(
                    "assets/sound/rowing.wav",
                    SoundOptions::new().looping(true).volume(0.3).fade_in(0.15),
                ));
            }
        } else if let Some(handle) = self.rowing.take() {
            handle.fade_out(0.25);
        }

        if let Some(player) = canvas.get_game_object_mut("2_player") {
            if right {
                if self.facing_left {
                    self.facing_left = false;
                    player.set_image(assets.player_right.clone());
                }
                player.position.0 = (player.position.0 + self.move_speed).min(GAME_WIDTH - self.player_w);
            } else if left {
                if !self.facing_left {
                    self.facing_left = true;
                    player.set_image(assets.player_left.clone());
                }
                player.position.0 = (player.position.0 - self.move_speed).max(0.0);
            }
        }
    }

    pub fn handle_reeling_sound(&mut self, canvas: &mut Canvas, cast: bool) {
        let line_is_moving = (cast && self.line_len < self.max_line)
            || (!cast && self.line_len > 0.0);

        if line_is_moving {
            let done = self.reeling.as_ref().map_or(true, |h| h.is_finished());
            if done {
                self.reeling = Some(canvas.play_sound_with(
                    "assets/sound/reeling.mp3",
                    SoundOptions::new().looping(true).volume(0.1).fade_in(0.05),
                ));
            }
        } else if let Some(handle) = self.reeling.take() {
            handle.fade_out(0.1);
        }
    }

    pub fn update_line(&mut self, cast: bool) {
        if cast {
            self.line_len = (self.line_len + self.line_speed).min(self.max_line);
        } else {
            self.line_len = (self.line_len - self.line_speed * 2.0).max(0.0);
        }
    }

    // Returns (rod_x, rod_y, hook_x, hook_y)
    pub fn rod_and_hook(&self, player_pos: (f32, f32)) -> (f32, f32, f32, f32) {
        let tip = if self.facing_left { self.rod_tip_left } else { self.rod_tip_right };
        let rod_x = player_pos.0 + self.player_w * tip.0 - 2.0;
        let rod_y = player_pos.1 + self.player_h * tip.1;
        let hook_x = rod_x - self.hook_w / 2.0;
        let hook_y = rod_y + self.line_len - 6.0;
        (rod_x, rod_y, hook_x, hook_y)
    }

    pub fn draw_line_and_hook(&mut self, canvas: &mut Canvas, assets: &GameAssets,
                               rod_x: f32, rod_y: f32, hx: f32, hy: f32) {
        if self.line_len > 0.0 {
            let hook_img = if self.facing_left {
                assets.hook_left.clone()
            } else {
                assets.hook_right.clone()
            };

            if let Some(line) = canvas.get_game_object_mut("fishing_line") {
                line.position = (rod_x, rod_y);
                line.size = (4.0, self.line_len);
                // Only rebuild the line image when the length changes noticeably
                if (self.line_len - self.last_line).abs() > 1.0 {
                    line.set_image(line_image(self.line_len));
                    self.last_line = self.line_len;
                }
            } else {
                canvas.add_game_object("fishing_line".into(),
                    GameObject::build("fishing_line")
                        .image(line_image(self.line_len))
                        .size(4.0, self.line_len)
                        .position(rod_x, rod_y)
                        .finish());
                self.last_line = self.line_len;
            }

            if let Some(hook) = canvas.get_game_object_mut("fishing_hook") {
                hook.position = (hx, hy);
                hook.set_image(hook_img);
            } else {
                canvas.add_game_object("fishing_hook".into(),
                    GameObject::build("fishing_hook")
                        .image(hook_img)
                        .size(self.hook_w, self.hook_h)
                        .position(hx, hy)
                        .finish());
            }
        } else {
            canvas.remove_game_object("fishing_line");
            canvas.remove_game_object("fishing_hook");
            self.last_line = -1.0;
        }
    }
}