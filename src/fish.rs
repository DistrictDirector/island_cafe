use quartz::{load_image_sized, rotate_ccw, GameObject, Image, ShapeType};
use quartz::entropy::Entropy;

const GAME_WIDTH: f32 = 3200.0;

#[derive(Clone, Copy, PartialEq)]
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
    pub gold_hooked:  Image,
    pub blue_hooked:  Image,
}

impl GameAssets {
    pub fn load(player_w: f32, player_h: f32, hook_w: f32, hook_h: f32, fish_w: f32, fish_h: f32) -> Self {
        let gold_right = load_image_sized("assets/fish/goldfishright.png", fish_w, fish_h);
        let blue_right = load_image_sized("assets/fish/bluefishright.png", fish_w, fish_h);
        Self {
            player_left:  load_image_sized("assets/player/playerleft.png",  player_w, player_h),
            player_right: load_image_sized("assets/player/playerright.png", player_w, player_h),
            hook_left:    load_image_sized("assets/player/hookleft.png",    hook_w,   hook_h),
            hook_right:   load_image_sized("assets/player/hookright.png",   hook_w,   hook_h),
            gold_left:    load_image_sized("assets/fish/goldfishleft.png",  fish_w,   fish_h),
            gold_right:   gold_right.clone(),
            blue_left:    load_image_sized("assets/fish/bluefishleft.png",  fish_w,   fish_h),
            blue_right:   blue_right.clone(),
            gold_hooked:  rotate_ccw(gold_right),
            blue_hooked:  rotate_ccw(blue_right),
        }
    }
}

#[derive(Clone)]
pub struct CatchAnimation {
    pub kind: FishKind,
    pub done: bool,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    peak_y: f32,
    progress: f32, 
    spin_angle: f32,
}

impl CatchAnimation {
    pub fn new(x: f32, y: f32, width: f32, height: f32, kind: FishKind, player_x: f32, player_y: f32) -> Self {
        Self {
            kind, done: false,
            x, y, width, height,
            start_x: x, start_y: y,
            end_x: player_x, end_y: player_y,
            peak_y: y.min(player_y) - 400.0,
            progress: 0.0,
            spin_angle: 0.0,
        }
    }

    pub fn update(&mut self) {
        self.progress = (self.progress + 0.025).min(1.0);

        let p = self.progress;
        let inverse = 1.0 - p;
        let middle_x = (self.start_x + self.end_x) / 2.0;

        // Bezier arc — blends start, peak, and end smoothly
        self.x = inverse*inverse*self.start_x + 2.0*inverse*p*middle_x  + p*p*self.end_x;
        self.y = inverse*inverse*self.start_y + 2.0*inverse*p*self.peak_y + p*p*self.end_y;

        self.spin_angle += 0.12 + p * 0.25;

        if self.progress >= 1.0 { self.done = true; }
    }

    pub fn image(&self, assets: &GameAssets) -> Image {
        let base = match self.kind { FishKind::Gold => &assets.gold_right, FishKind::Blue => &assets.blue_right };
        let mut img = base.clone();
        img.shape = ShapeType::Rectangle(0.0, (self.width, self.height), self.spin_angle);
        img
    }
}

// A fish stuck on the hook being reeled in
#[derive(Clone)]
pub struct HookedFish {
    pub id: usize,
    pub width: f32,
    pub height: f32,
    pub kind: FishKind,
    pub rotated_width: f32,   // width and height swap after rotating 90 degrees
    pub rotated_height: f32,
}

impl HookedFish {
    pub fn name(&self) -> String { format!("hooked_{}", self.id) }

    pub fn image(&self, assets: &GameAssets) -> Image {
        match self.kind { FishKind::Gold => assets.gold_hooked.clone(), FishKind::Blue => assets.blue_hooked.clone() }
    }
}

// A fish swimming freely in the water
#[derive(Clone)]
pub struct Fish {
    pub id: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub speed: f32,
    pub facing_left: bool,
    pub kind: FishKind,
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

    pub fn touches_hook(&self, hook_x: f32, hook_y: f32, hook_w: f32, hook_h: f32) -> bool {
        self.x < hook_x + hook_w && self.x + self.width  > hook_x
        && self.y < hook_y + hook_h && self.y + self.height > hook_y
    }
}

#[derive(Clone)]
pub struct FishManager {
    pub fish:          Vec<Fish>,
    pub hooked:        Option<HookedFish>,
    pub catch_anim:    Option<CatchAnimation>,
    pub just_launched: bool,
    pub last_caught:   Option<(FishKind, f32)>,
    spawn_from_left:   bool,
    spawn_timer:       f32,
    next_id:           usize, 
    rng:               Entropy,
}

impl FishManager {
    pub fn new() -> Self {
        Self {
            fish: Vec::new(), hooked: None, catch_anim: None,
            just_launched: false, last_caught: None,
            spawn_from_left: true, spawn_timer: 0.0, next_id: 0,
            rng: Entropy::from_seed(98765432101234567),
        }
    }

    pub fn update(
        &mut self, canvas: &mut quartz::Canvas, assets: &GameAssets,
        hook_x: f32, hook_y: f32, hook_w: f32, hook_h: f32,
        line_active: bool, line_len: f32,
        player_x: f32, player_y: f32, player_h: f32,
    ) {
        self.just_launched = false;
        self.last_caught = None;

        // Step 1 — move the flying catch animation forward
        if let Some(anim) = self.catch_anim.as_mut() { anim.update(); }
        if let Some(anim) = &self.catch_anim {
            if anim.done {
                self.last_caught = Some((anim.kind, anim.width));
                canvas.remove_game_object("catch_anim");
                self.catch_anim = None;
            } else {
                let img = anim.image(assets);
                let pos = (anim.x, anim.y);
                if let Some(obj) = canvas.get_game_object_mut("catch_anim") {
                    obj.position = pos;
                    obj.set_image(img);
                } else {
                    let anim = self.catch_anim.as_ref().unwrap();
                    canvas.add_game_object("catch_anim".into(),
                        GameObject::build("catch_anim")
                            .image(anim.image(assets))
                            .size(anim.width, anim.height)
                            .position(anim.x, anim.y)
                            .finish());
                }
            }
        }

        // Step 2 — move the hooked fish along with the hook
        let should_launch = if let Some(ref hooked) = self.hooked {
            let fish_x = hook_x - hooked.width / 2.0;
            let player_reached = hook_y <= player_y + player_h + 20.0 || line_len < 10.0;

            if player_reached {
                canvas.remove_game_object(&hooked.name());
                true
            } else {
                let img = hooked.image(assets);
                if let Some(obj) = canvas.get_game_object_mut(&hooked.name()) {
                    obj.position = (fish_x, hook_y);
                    obj.set_image(img);
                } else {
                    canvas.add_game_object(hooked.name(),
                        GameObject::build(hooked.name())
                            .image(hooked.image(assets))
                            .size(hooked.rotated_width, hooked.rotated_height)
                            .position(fish_x, hook_y)
                            .finish());
                }
                false
            }
        } else { false };

        if should_launch {
            if let Some(hooked) = self.hooked.take() {
                let fish_x = hook_x - hooked.width / 2.0;
                self.catch_anim = Some(CatchAnimation::new(
                    fish_x, hook_y, hooked.width, hooked.height, hooked.kind, player_x, player_y,
                ));
                self.just_launched = true;
            }
        }

        // Step 3 — move all free fish and check if the hook catches one
        let mut fish_to_hook: Option<usize> = None;

        for fish in &mut self.fish {
            let was_facing_left = fish.facing_left;

            if fish.facing_left { fish.x -= fish.speed; } else { fish.x += fish.speed; }

            if fish.x + fish.width > GAME_WIDTH { fish.x = GAME_WIDTH - fish.width; fish.facing_left = true; }
            if fish.x < 0.0 { fish.x = 0.0; fish.facing_left = false; }

            let line_is_in_water = line_active && self.hooked.is_none();
            if line_is_in_water && fish.touches_hook(hook_x, hook_y, hook_w, hook_h) {
                fish_to_hook = Some(fish.id);
                continue;
            }

            if let Some(obj) = canvas.get_game_object_mut(&fish.name()) {
                obj.position = (fish.x, fish.y);
                if fish.facing_left != was_facing_left {
                    obj.set_image(fish.image(assets).clone());
                }
            }
        }

        if let Some(id) = fish_to_hook {
            if let Some(fish) = self.fish.iter().find(|f| f.id == id) {
                self.hooked = Some(HookedFish {
                    id: fish.id, kind: fish.kind,
                    width: fish.width, height: fish.height,
                    rotated_width: fish.height,  // 90 degree rotation swaps width and height
                    rotated_height: fish.width,
                });
                canvas.remove_game_object(&fish.name());
                self.fish.retain(|f| f.id != id);
            }
        }

        // Step 4 — spawn a new fish every so often if the water isn't full
        self.spawn_timer += 1.0;
        let wait_time = 120.0 + self.rng.range(0.0, 180.0);
        let water_is_not_full = self.fish.len() < 8;

        if self.spawn_timer >= wait_time && water_is_not_full {
            self.spawn_timer = 0.0;

            let width  = 60.0 + self.rng.range(0.0, 60.0);
            let height = width * 0.55;
            let depth  = 950.0 + self.rng.range(0.0, 900.0);
            let start_x = if self.spawn_from_left { -width } else { GAME_WIDTH };
            let kind = if self.spawn_from_left { FishKind::Gold } else { FishKind::Blue };
            let facing_left = !self.spawn_from_left;

            let fish = Fish {
                id: self.next_id,
                x: start_x, y: depth,
                width, height,
                speed: self.rng.range(2.0, 6.0),
                facing_left, kind,
            };

            canvas.add_game_object(fish.name(),
                GameObject::build(fish.name())
                    .image(fish.image(assets).clone())
                    .size(fish.width, fish.height)
                    .position(fish.x, fish.y)
                    .finish());

            self.fish.push(fish);
            self.next_id += 1;
            self.spawn_from_left = !self.spawn_from_left;
        }
    }
}