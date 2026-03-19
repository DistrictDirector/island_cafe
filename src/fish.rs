use quartz::{load_image_sized, GameObject, Image, ShapeType};

pub fn next_rand(seed: &mut u64) -> f32 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*seed >> 33) as f32) / (u32::MAX as f32)
}

#[derive(Clone, Copy)]
pub enum FishKind { Gold, Blue }

#[derive(Clone)]
pub struct GameAssets {
    pub player_left:  Image,
    pub player_right: Image,
    pub hook_left:    Image,
    pub hook_right:   Image,
    pub gold_left:    Image,
    pub gold_right:   Image,
    pub blue_left:    Image,
    pub blue_right:   Image,
}

impl GameAssets {
    pub fn load(player_w: f32, player_h: f32, hook_w: f32, hook_h: f32, fish_w: f32, fish_h: f32) -> Self {
        Self {
            player_left:  load_image_sized("assets/playerleft.png",    player_w, player_h),
            player_right: load_image_sized("assets/playerright.png",   player_w, player_h),
            hook_left:    load_image_sized("assets/hookleft.png",      hook_w,   hook_h),
            hook_right:   load_image_sized("assets/hookright.png",     hook_w,   hook_h),
            gold_left:    load_image_sized("assets/goldfishleft.png",  fish_w,   fish_h),
            gold_right:   load_image_sized("assets/goldfishright.png", fish_w,   fish_h),
            blue_left:    load_image_sized("assets/bluefishleft.png",  fish_w,   fish_h),
            blue_right:   load_image_sized("assets/bluefishright.png", fish_w,   fish_h),
        }
    }
}

#[derive(Clone)]
pub struct CatchAnimation {
    pub x:        f32,
    pub y:        f32,
    pub w:        f32,
    pub h:        f32,
    pub kind:     FishKind,
    pub target_x: f32,
    pub target_y: f32,
    pub boat_y:   f32,
    vel_x:        f32,
    vel_y:        f32,
    gravity:      f32,
    spin:         f32,
    angle:        f32,
    pub done:     bool,
    pub missed:   bool,
}

impl CatchAnimation {
    pub fn new(
        fish_x:      f32,
        fish_y:      f32,
        fish_w:      f32,
        fish_h:      f32,
        kind:        FishKind,
        player_x:    f32,
        player_y:    f32,
        boat_bottom: f32,
    ) -> Self {
        let dx     = player_x - fish_x;
        let dy     = player_y - fish_y;
        let frames = 35.0;
        Self {
            x: fish_x, y: fish_y, w: fish_w, h: fish_h, kind,
            target_x: player_x,
            target_y: player_y,
            boat_y:   boat_bottom,
            vel_x:    dx / frames,
            vel_y:    dy / frames - 10.0,
            gravity:  0.45,
            spin:     0.22,
            angle:    0.0,
            done:     false,
            missed:   false,
        }
    }

    pub fn image(&self, assets: &GameAssets) -> Image {
        let base = match self.kind {
            FishKind::Gold => &assets.gold_right,
            FishKind::Blue => &assets.blue_right,
        };
        let mut img = base.clone();
        img.shape = ShapeType::Rectangle(0.0, (self.w, self.h), self.angle);
        img
    }

    pub fn update(&mut self) {
        self.x     += self.vel_x;
        self.y     += self.vel_y;
        self.vel_y += self.gravity;
        self.angle += self.spin;

        let dx = self.target_x - self.x;
        let dy = self.target_y - self.y;
        if (dx * dx + dy * dy).sqrt() < 60.0 {
            self.done = true;
            return;
        }

        if self.y > self.boat_y + 80.0 {
            self.missed = true;
        }
    }
}

#[derive(Clone)]
pub struct HookedFish {
    pub id:   usize,
    pub w:    f32,
    pub h:    f32,
    pub kind: FishKind,
}

impl HookedFish {
    pub fn name(&self) -> String { format!("hooked_{}", self.id) }

    pub fn image<'a>(&self, assets: &'a GameAssets) -> &'a Image {
        match self.kind {
            FishKind::Gold => &assets.gold_right,
            FishKind::Blue => &assets.blue_right,
        }
    }
}

#[derive(Clone)]
pub struct Fish {
    pub id:          usize,
    pub x:           f32,
    pub y:           f32,
    pub w:           f32,
    pub h:           f32,
    pub speed:       f32,
    pub facing_left: bool,
    pub kind:        FishKind,
}

impl Fish {
    pub fn name(&self) -> String { format!("fish_{}", self.id) }

    pub fn image<'a>(&self, assets: &'a GameAssets) -> &'a Image {
        match (self.kind, self.facing_left) {
            (FishKind::Gold, true)  => &assets.gold_left,
            (FishKind::Gold, false) => &assets.gold_right,
            (FishKind::Blue, true)  => &assets.blue_left,
            (FishKind::Blue, false) => &assets.blue_right,
        }
    }

    pub fn overlaps_hook(&self, hx: f32, hy: f32, hw: f32, hh: f32) -> bool {
        self.x < hx + hw && self.x + self.w > hx &&
        self.y < hy + hh && self.y + self.h > hy
    }
}

#[derive(Clone)]
pub struct FishManager {
    pub fish:       Vec<Fish>,
    pub hooked:     Option<HookedFish>,
    pub catch_anim: Option<CatchAnimation>,
    next_id:        usize,
    spawn_timer:    f32,
    pub score:      u32,
    rng:            u64,
}

impl FishManager {
    pub fn new() -> Self {
        Self {
            fish:        Vec::new(),
            hooked:      None,
            catch_anim:  None,
            next_id:     0,
            spawn_timer: 0.0,
            score:       0,
            rng:         98765432101234567,
        }
    }

    pub fn is_busy(&self) -> bool {
        self.hooked.is_some() || self.catch_anim.is_some()
    }

    pub fn update(
        &mut self,
        canvas:      &mut quartz::Canvas,
        assets:      &GameAssets,
        hook_x:      f32,
        hook_y:      f32,
        hook_w:      f32,
        hook_h:      f32,
        line_active: bool,
        line_len:    f32,
        player_x:    f32,
        player_y:    f32,
        player_h:    f32,
    ) {
        if let Some(ref mut anim) = self.catch_anim {
            anim.update();
            if anim.done {
                canvas.remove_game_object("catch_anim");
                self.score += 1;
                println!("Score: {}", self.score);
                self.catch_anim = None;
            } else if anim.missed {
                println!("Fish got away!");
                let escaped = Fish {
                    id:          self.next_id,
                    x:           anim.x,
                    y:           anim.boat_y + 40.0,
                    w:           anim.w,
                    h:           anim.h,
                    speed:       3.0,
                    facing_left: anim.x > player_x,
                    kind:        anim.kind,
                };
                canvas.remove_game_object("catch_anim");
                canvas.add_game_object(
                    escaped.name(),
                    GameObject::build(escaped.name())
                        .image(escaped.image(assets).clone())
                        .size(escaped.w, escaped.h)
                        .position(escaped.x, escaped.y)
                        .finish(),
                );
                self.fish.push(escaped);
                self.next_id += 1;
                self.catch_anim = None;
            } else {
                let img = anim.image(assets);
                if let Some(obj) = canvas.get_game_object_mut("catch_anim") {
                    obj.position = (anim.x, anim.y);
                    obj.set_image(img);
                } else {
                    canvas.add_game_object(
                        "catch_anim".to_string(),
                        GameObject::build("catch_anim")
                            .image(anim.image(assets))
                            .size(anim.w, anim.h)
                            .position(anim.x, anim.y)
                            .finish(),
                    );
                }
            }
        }

        let mut should_launch = false;
        let mut launch_x      = 0.0;
        let mut launch_y      = 0.0;

        if let Some(ref hooked) = self.hooked {
            let fish_x    = hook_x - hooked.w / 2.0;
            let fish_y    = hook_y;
            let trigger_y = player_y + player_h + 20.0;

            if fish_y <= trigger_y || line_len < 10.0 {
                canvas.remove_game_object(&hooked.name());
                should_launch = true;
                launch_x      = fish_x;
                launch_y      = fish_y;
            } else {
                if let Some(obj) = canvas.get_game_object_mut(&hooked.name()) {
                    obj.position = (fish_x, fish_y);
                    obj.set_image(hooked.image(assets).clone());
                } else {
                    canvas.add_game_object(
                        hooked.name(),
                        GameObject::build(hooked.name())
                            .image(hooked.image(assets).clone())
                            .size(hooked.w, hooked.h)
                            .position(fish_x, fish_y)
                            .finish(),
                    );
                }
            }
        }

        if should_launch {
            if let Some(hooked) = self.hooked.take() {
                self.catch_anim = Some(CatchAnimation::new(
                    launch_x, launch_y,
                    hooked.w, hooked.h,
                    hooked.kind,
                    player_x, player_y,
                    player_y + player_h,
                ));
            }
        }

        self.spawn_timer += 1.0;
        let spawn_interval = 120.0 + next_rand(&mut self.rng) * 180.0;
        if self.spawn_timer >= spawn_interval && self.fish.len() < 8 {
            self.spawn_timer = 0.0;
            self.spawn(canvas, assets);
        }

        let mut hooked_id: Option<usize> = None;

        for fish in &mut self.fish {
            if fish.facing_left { fish.x -= fish.speed; } else { fish.x += fish.speed; }

            if fish.x + fish.w > 3840.0 {
                fish.x = 3840.0 - fish.w;
                fish.facing_left = true;
            } else if fish.x < 0.0 {
                fish.x = 0.0;
                fish.facing_left = false;
            }

            if line_active && self.hooked.is_none() && fish.overlaps_hook(hook_x, hook_y, hook_w, hook_h) {
                hooked_id = Some(fish.id);
                continue;
            }

            if let Some(obj) = canvas.get_game_object_mut(&fish.name()) {
                obj.position = (fish.x, fish.y);
                obj.set_image(fish.image(assets).clone());
            }
        }

        if let Some(id) = hooked_id {
            if let Some(fish) = self.fish.iter().find(|f| f.id == id) {
                let hooked = HookedFish { id: fish.id, w: fish.w, h: fish.h, kind: fish.kind };
                canvas.remove_game_object(&fish.name());
                self.fish.retain(|f| f.id != id);
                self.hooked = Some(hooked);
                println!("Fish hooked! Reel it in!");
            }
        }
    }

    fn spawn(&mut self, canvas: &mut quartz::Canvas, assets: &GameAssets) {
        let from_left   = self.next_id % 2 == 0;
        let fish_w      = 60.0 + next_rand(&mut self.rng) * 60.0;
        let fish_h      = fish_w * 0.55;
        let fish_y      = 950.0 + next_rand(&mut self.rng) * 900.0;
        let fish_x      = if from_left { -fish_w } else { 3840.0 };
        let speed       = 2.0 + next_rand(&mut self.rng) * 4.0;
        let kind        = if self.next_id % 2 == 0 { FishKind::Gold } else { FishKind::Blue };
        let facing_left = !from_left;

        let fish = Fish { id: self.next_id, x: fish_x, y: fish_y, w: fish_w, h: fish_h, speed, facing_left, kind };

        canvas.add_game_object(
            fish.name(),
            GameObject::build(fish.name())
                .image(fish.image(assets).clone())
                .size(fish.w, fish.h)
                .position(fish.x, fish.y)
                .finish(),
        );

        self.fish.push(fish);
        self.next_id += 1;
    }
}