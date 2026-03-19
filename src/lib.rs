use flowmango::prelude::*;
use quartz::{load_image, load_image_sized, GameObject};
use image::{RgbaImage, Rgba};
use prism::canvas::{Image, ShapeType};
use ramp::prism;

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
        let move_speed = 10.0;
        let max_line   = 550.0;
        let line_speed = 14.0;
        let hook_w     = 32.0;
        let hook_h     = 38.0;

        let rod_tip_right = (0.85, 0.20);
        let rod_tip_left  = (0.15, 0.20);

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
                .image(load_image_sized("assets/playerright.png", player_w, player_h))
                .size(player_w, player_h)
                .position((3840.0 - player_w) / 2.0, player_y)
                .tag("player")
                .layer(2)
                .build(ctx)
        );

        scene.set_camera_follow("player".into(), 0.10);

        if let Some(layer) = scene.get_layer_mut(LayerId(2)) {
            layer.canvas_mut().play_sound_with(
                "assets/backgroundmusic.mp3",
                SoundOptions::new().looping(true).volume(0.1).fade_in(1.0),
            );

            let mut rowing:     Option<SoundHandle> = None;
            let mut line_len:   f32  = 0.0;
            let mut facing_left = false;

            layer.canvas_mut().on_update(move |canvas| {
                let left   = canvas.key("left");
                let right  = canvas.key("right");
                let cast   = canvas.key("up");
                let moving = left || right;

                if moving && rowing.as_ref().map_or(true, |h| h.is_finished()) {
                    rowing = Some(canvas.play_sound_with(
                        "assets/rowing.wav",
                        SoundOptions::new().looping(true).volume(0.85).fade_in(0.15),
                    ));
                }
                if !moving {
                    if let Some(h) = rowing.take() { h.fade_out(0.25); }
                }

                if let Some(go) = canvas.get_game_object_mut("2_player") {
                    if right {
                        facing_left = false;
                        go.position.0 = (go.position.0 + move_speed).min(3840.0 - player_w);
                        go.set_image(load_image_sized("assets/playerright.png", player_w, player_h));
                    } else if left {
                        facing_left = true;
                        go.position.0 = (go.position.0 - move_speed).max(0.0);
                        go.set_image(load_image_sized("assets/playerleft.png", player_w, player_h));
                    }
                }

                line_len = if cast {
                    (line_len + line_speed).min(max_line)
                } else {
                    (line_len - line_speed * 2.0).max(0.0)
                };

                if line_len > 0.0 {
                    let (px, py) = canvas.get_game_object("2_player")
                        .map(|g| g.position)
                        .unwrap_or((0.0, 0.0));

                    let tip = if facing_left { rod_tip_left } else { rod_tip_right };
                    let lx  = px + player_w * tip.0 - 2.0;
                    let ly  = py + player_h * tip.1;

                    if let Some(line) = canvas.get_game_object_mut("fishing_line") {
                        line.position = (lx, ly);
                        line.set_image(line_image(line_len));
                        line.size = (4.0, line_len);
                    } else {
                        canvas.add_game_object(
                            "fishing_line".to_string(),
                            GameObject::build("fishing_line")
                                .image(line_image(line_len))
                                .size(4.0, line_len)
                                .position(lx, ly)
                                .finish(),
                        );
                    }

                    let hx = lx - hook_w / 2.0;
                    let hy = ly + line_len - 6.0;

                    let hook_img = if facing_left {
                        load_image_sized("assets/hookleft.png", hook_w, hook_h)
                    } else {
                        load_image_sized("assets/hookright.png", hook_w, hook_h)
                    };

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
                }
            });
        }

        scene
    }
}

ramp::run! { |ctx: &mut Context, _assets: Assets| {
    IslandCafe::new(ctx)
}}