mod fish;
mod player;
mod sidebar;

use flowmango::prelude::*;
use quartz::{load_image, load_image_sized, GameObject, Font};
use quartz::entropy::Entropy;
use ramp::prism;

use fish::{GameAssets, FishManager};
use player::Player;
use sidebar::{Sidebar, GAME_WIDTH};

pub struct IslandCafe;

// ---------------------------------------------------------------------------
// Game — holds all state that needs to live across ticks
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Game {
    assets: GameAssets,
    player: Player,
    fish_manager: FishManager,
    sidebar: Sidebar,
}

impl Game {
    fn new(assets: GameAssets, player_w: f32, player_h: f32,
           hook_w: f32, hook_h: f32, sidebar: Sidebar) -> Self {
        Self {
            assets,
            player: Player::new(player_w, player_h, hook_w, hook_h),
            fish_manager: FishManager::new(),
            sidebar,
        }
    }

    fn tick(&mut self, canvas: &mut Canvas) {
        let left = canvas.key("left");
        let right = canvas.key("right");
        let cast = canvas.key("up");

        // Stop rowing sound while a catch animation is playing
        if self.fish_manager.catch_anim.is_some() {
            self.player.stop_rowing();
        } else {
            self.player.handle_movement(canvas, &self.assets, left, right);
        }

        self.player.handle_reeling_sound(canvas, cast);
        self.player.update_line(cast);

        let player_pos = canvas.get_game_object("2_player")
            .map(|g| g.position)
            .unwrap_or((0.0, 0.0));

        let (rod_x, rod_y, hx, hy) = self.player.rod_and_hook(player_pos);
        let player_cx = player_pos.0 + self.player.player_w / 2.0;
        let player_cy = player_pos.1 + self.player.player_h / 2.0;

        self.player.draw_line_and_hook(canvas, &self.assets, rod_x, rod_y, hx, hy);

        self.fish_manager.update(
            canvas,
            &self.assets,
            hx, hy,
            self.player.hook_w,
            self.player.hook_h,
            self.player.line_len > 0.0,
            self.player.line_len,
            player_cx, player_cy,
            self.player.player_h,
        );

        // If a fish just landed, check it against the current order
        if let Some((kind, w)) = self.fish_manager.last_caught.take() {
            self.sidebar.check_catch(kind, w);
        }

        if self.fish_manager.just_launched {
            canvas.play_sound("assets/sound/splash.mp3");
        }

        self.sidebar.update(canvas, &self.assets);
    }
}

// ---------------------------------------------------------------------------
// IslandCafe — builds the scene on startup
// ---------------------------------------------------------------------------

impl IslandCafe {
    pub fn new(ctx: &mut Context) -> Scene {
        let player_w = 200.0;
        let player_h = 200.0;
        let player_y = 700.0;
        let hook_w = 32.0;
        let hook_h = 38.0;
        let fish_w = 90.0;
        let fish_h = fish_w * 0.55;

        let font_bytes = std::fs::read("assets/fonts/fontbold.ttf")
            .expect("missing assets/fonts/fontbold.ttf");
        let font = Font::from_bytes(&font_bytes).unwrap();

        let assets = GameAssets::load(player_w, player_h, hook_w, hook_h, fish_w, fish_h);

        let mut scene = Scene::new(ctx, CanvasMode::Landscape, 3);

        scene.get_layer_mut(LayerId(0)).unwrap().set_parallax(0.3);
        scene.get_layer_mut(LayerId(1)).unwrap().set_parallax(0.7);
        scene.get_layer_mut(LayerId(2)).unwrap().set_parallax(1.0);

        // Static background layers
        scene.add_object(
            GameObject::build("sky_layer")
                .image(load_image("assets/layers/skylayer.png"))
                .size(3840.0, 2160.0).position(0.0, 0.0).layer(0).build(ctx)
        );
        scene.add_object(
            GameObject::build("island_layer")
                .image(load_image("assets/layers/islandlayer.png"))
                .size(3840.0, 2160.0).position(0.0, 0.0).layer(1).build(ctx)
        );

        // Clouds that drift slowly across the sky
        let canvas_w = 3840.0_f32;
        let spacing = 640.0_f32;
        let count = (canvas_w / spacing).ceil() as usize + 1;
        let mut rng = Entropy::from_seed(12345);
        let mut speeds = Vec::new();
        let mut widths = Vec::new();

        for i in 0..count {
            let w = 380.0 + rng.range(0.0, 400.0);
            let h = w * 0.42;
            let x = i as f32 * spacing + rng.range(-150.0, 150.0);
            let y = 20.0 + rng.range(0.0, 180.0);
            let spd = rng.range(0.18, 0.53);
            speeds.push(spd);
            widths.push(w);
            scene.add_object(
                GameObject::build(format!("cloud_{}", i))
                    .image(load_image_sized("assets/layers/cloud.png", w, h))
                    .size(w, h).position(x, y).layer(0).build(ctx)
            );
        }

        // Player starts in the middle of the game area (not the full 3840px canvas)
        scene.add_object(
            GameObject::build("player")
                .image(assets.player_right.clone())
                .size(player_w, player_h)
                .position((GAME_WIDTH - player_w) / 2.0, player_y)
                .tag("player").layer(2).build(ctx)
        );

        // Start background music looping on layer 2
        scene.get_layer_mut(LayerId(2)).unwrap().canvas_mut().play_sound_with(
            "assets/sound/backgroundmusic.mp3",
            SoundOptions::new().looping(true).volume(0.5).fade_in(1.0),
        );

        // Set up the sidebar on layer 2
        let mut sidebar = Sidebar::new(font.clone());
        sidebar.setup(scene.get_layer_mut(LayerId(2)).unwrap().canvas_mut(), &assets);

        // Cloud movement runs on layer 0
        scene.get_layer_mut(LayerId(0)).unwrap().canvas_mut().on_update(move |canvas| {
            for i in 0..count {
                let name = format!("0_cloud_{}", i);
                let speed = speeds[i];
                let w = widths[i];
                if let Some(cloud) = canvas.get_game_object_mut(&name) {
                    cloud.position.0 += speed;
                    if cloud.position.0 > canvas_w {
                        cloud.position.0 = -w;
                    }
                }
            }
        });

        // Main game logic runs on layer 2
        let mut game = Game::new(assets.clone(), player_w, player_h, hook_w, hook_h, sidebar);
        scene.get_layer_mut(LayerId(2)).unwrap().canvas_mut().on_update(move |canvas| {
            game.tick(canvas);
        });

        scene
    }
}

ramp::run! { |ctx: &mut Context, _assets: Assets| {
    IslandCafe::new(ctx)
}}