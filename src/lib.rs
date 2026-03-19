mod fish;

use flowmango::prelude::*;
use quartz::{load_image, GameObject, Image, ShapeType};
use image::{RgbaImage, Rgba};
use ramp::prism;

use fish::{GameAssets, FishManager};

pub struct IslandCafe;

fn line_image(h: f32) -> Image {
    let img = RgbaImage::from_pixel(1, 1, Rgba([101, 67, 33, 255]));
    Image { shape: ShapeType::Rectangle(0.0, (4.0, h), 0.0), image: img.into(), color: None }
}

impl IslandCafe {
    pub fn new(ctx: &mut Context) -> Scene {
        let player_w   = 200.0;
        let player_h   = 200.0;
        let player_y   = 700.0;
        let move_speed = 6.0;
        let max_line   = 550.0;
        let line_speed = 8.0;
        let hook_w     = 32.0;
        let hook_h     = 38.0;
        let fish_w     = 90.0;
        let fish_h     = fish_w * 0.55;

        let rod_tip_right = (0.85, 0.20);
        let rod_tip_left  = (0.15, 0.20);

        let assets = GameAssets::load(player_w, player_h, hook_w, hook_h, fish_w, fish_h);

        let mut scene = Scene::new(ctx, CanvasMode::Landscape, 3);

        scene.get_layer_mut(LayerId(0)).unwrap().set_parallax(0.3);
        scene.get_layer_mut(LayerId(1)).unwrap().set_parallax(0.7);
        scene.get_layer_mut(LayerId(2)).unwrap().set_parallax(1.0);

        scene.add_object(
            GameObject::build("first_layer")
                .image(load_image("assets/firstlayer.png"))
                .size(3840.0, 2160.0)
                .position(0.0, 0.0)
                .layer(0)
                .build(ctx)
        );

        scene.add_object(
            GameObject::build("player")
                .image(assets.player_right.clone())
                .size(player_w, player_h)
                .position((3840.0 - player_w) / 2.0, player_y)
                .tag("player")
                .layer(2)
                .build(ctx)
        );

        scene.set_camera_follow("player".into(), 0.10);

        scene.get_layer_mut(LayerId(2)).unwrap().canvas_mut().play_sound_with(
            "assets/backgroundmusic.mp3",
            SoundOptions::new().looping(true).volume(0.1).fade_in(1.0),
        );

        let mut rowing:      Option<SoundHandle> = None;
        let mut line_len:    f32  = 0.0;
        let mut last_line:   f32  = -1.0;
        let mut facing_left       = false;
        let mut fish_manager      = FishManager::new();

        scene.get_layer_mut(LayerId(2)).unwrap().canvas_mut().on_update(move |canvas| {
            let left  = canvas.key("left");
            let right = canvas.key("right");
            let cast  = canvas.key("up");

            let catch_playing = fish_manager.catch_anim.is_some();

            if !catch_playing {
                let moving = left || right;
                if moving {
                    let sound_done = rowing.as_ref().map_or(true, |h| h.is_finished());
                    if sound_done {
                        rowing = Some(canvas.play_sound_with(
                            "assets/rowing.wav",
                            SoundOptions::new().looping(true).volume(0.85).fade_in(0.15),
                        ));
                    }
                } else {
                    if let Some(h) = rowing.take() { h.fade_out(0.25); }
                }

                if let Some(player) = canvas.get_game_object_mut("2_player") {
                    if right {
                        if facing_left {
                            facing_left = false;
                            player.set_image(assets.player_right.clone());
                        }
                        player.position.0 = (player.position.0 + move_speed).min(3840.0 - player_w);
                    } else if left {
                        if !facing_left {
                            facing_left = true;
                            player.set_image(assets.player_left.clone());
                        }
                        player.position.0 = (player.position.0 - move_speed).max(0.0);
                    }
                }
            } else {
                if let Some(h) = rowing.take() { h.fade_out(0.25); }
            }

            if cast {
                line_len = (line_len + line_speed).min(max_line);
            } else {
                line_len = (line_len - line_speed * 2.0).max(0.0);
            }

            let player_pos = canvas.get_game_object("2_player")
                .map(|g| g.position)
                .unwrap_or((0.0, 0.0));
            let tip    = if facing_left { rod_tip_left } else { rod_tip_right };
            let rod_x  = player_pos.0 + player_w * tip.0 - 2.0;
            let rod_y  = player_pos.1 + player_h * tip.1;
            let hx     = rod_x - hook_w / 2.0;
            let hy     = rod_y + line_len - 6.0;

            let player_center_x = player_pos.0 + player_w / 2.0;
            let player_center_y = player_pos.1 + player_h / 2.0;

            if line_len > 0.0 {
                let hook_img = if facing_left {
                    assets.hook_left.clone()
                } else {
                    assets.hook_right.clone()
                };

                if let Some(line) = canvas.get_game_object_mut("fishing_line") {
                    line.position = (rod_x, rod_y);
                    line.size     = (4.0, line_len);
                    if (line_len - last_line).abs() > 1.0 {
                        line.set_image(line_image(line_len));
                        last_line = line_len;
                    }
                } else {
                    canvas.add_game_object(
                        "fishing_line".to_string(),
                        GameObject::build("fishing_line")
                            .image(line_image(line_len))
                            .size(4.0, line_len)
                            .position(rod_x, rod_y)
                            .finish(),
                    );
                    last_line = line_len;
                }

                if let Some(hook) = canvas.get_game_object_mut("fishing_hook") {
                    hook.position = (hx, hy);
                    hook.set_image(hook_img);
                } else {
                    canvas.add_game_object(
                        "fishing_hook".to_string(),
                        GameObject::build("fishing_hook")
                            .image(hook_img)
                            .size(hook_w, hook_h)
                            .position(hx, hy)
                            .finish(),
                    );
                }
            } else {
                canvas.remove_game_object("fishing_line");
                canvas.remove_game_object("fishing_hook");
                last_line = -1.0;
            }

            fish_manager.update(
                canvas, &assets,
                hx, hy, hook_w, hook_h,
                line_len > 0.0,
                line_len,
                player_center_x, player_center_y,
                player_h,
            );
        });

        scene
    }
}

ramp::run! { |ctx: &mut Context, _assets: Assets| {
    IslandCafe::new(ctx)
}}