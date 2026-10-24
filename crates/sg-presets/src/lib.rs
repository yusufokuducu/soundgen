//! sg-presets â€” the factory sound library. One Rust function per effect.
//!
//! Design rules:
//!  * every recipe is a stack of 2-6 layers (transient / body / tail / sweetener)
//!  * `style_gain` decides how each layer behaves on the Realism <-> Cartoon axis
//!  * no samples, no external assets: everything is synthesised

pub mod ambience;
pub mod celebration;
pub mod crates;
pub mod foley;
pub mod gore;
pub mod loops;
pub mod ui;
pub mod weapons;
pub mod zombies;

use sg_engine::layer::Category;
use sg_engine::recipe::{MasterChain, Recipe};

/// Every factory preset, grouped by category.
pub fn all() -> Vec<Recipe> {
    let mut v = Vec::new();
    v.extend(weapons::presets());
    v.extend(zombies::presets());
    v.extend(gore::presets());
    v.extend(foley::presets());
    v.extend(loops::presets());
    v.extend(crates::presets());
    v.extend(celebration::presets());
    v.extend(ui::presets());
    v.extend(ambience::presets());
    v
}

/// Find a preset by id.
pub fn get(id: &str) -> Option<Recipe> {
    all().into_iter().find(|r| r.id == id)
}

/// Helper used by category modules.
pub fn base(id: &str, name: &str, cat: Category) -> Recipe {
    let mut r = Recipe::new(id, name, cat);
    r.master = MasterChain {
        target_peak_db: -1.0,
        ..Default::default()
    };
    r
}

/// Shared imports for every category module.
pub(crate) mod prelude {
    pub use crate::base;
    pub use sg_engine::layer::{Category, FilterEnv, Layer, LayerKind};
    pub use sg_engine::prelude::*;
    pub use sg_engine::recipe::{MasterChain, Recipe};
}


