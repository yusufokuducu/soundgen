//! sg-engine — recipe graph, renderer and audio analysis.

pub mod buffer;
pub mod layer;
pub mod recipe;
pub mod render;

pub use buffer::AudioBuffer;
pub use layer::{Category, FilterEnv, Layer, LayerKind};
pub use recipe::{MasterChain, Recipe};
pub use render::{render, render_variations, render_with_style};

pub mod prelude {
    pub use crate::layer::{Category, FilterEnv, Layer, LayerKind};
    pub use crate::recipe::{MasterChain, Recipe};
    pub use sg_dsp::env::AdEnv;
    pub use sg_dsp::modal::{inharmonic_modes, Mode};
    pub use sg_dsp::noise::NoiseColor;
    pub use sg_dsp::osc::WaveShape;
}
