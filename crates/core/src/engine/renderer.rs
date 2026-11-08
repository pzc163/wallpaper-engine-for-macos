use std::ffi::c_void;

use crate::{
    EngineError, WallpaperWindow,
    media::audio::AudioVolume,
    owe::backend::{FirstFrameCallback, OweBackend, OweScene},
    project::{ScalingMode, SceneDesc},
    web::{WebProject, WebScene},
};

pub(super) enum WallpaperRenderer {
    Native(OweScene),
    Web(WebScene),
}

macro_rules! forward {
    ($name:ident($($arg:ident: $ty:ty),*)) => {
        pub(super) fn $name(&mut self, $($arg:$ty),*) -> Result<(), EngineError> {
            match self {
                Self::Native(scene) => scene.$name($($arg),*),
                Self::Web(scene) => scene.$name($($arg),*),
            }
        }
    };
}

impl WallpaperRenderer {
    pub(super) fn open(
        backend: OweBackend,
        desc: &SceneDesc,
        window: &WallpaperWindow,
        scaling_mode: ScalingMode,
        scaling_factor: f64,
        resolution: Option<(u32, u32)>,
        callback: FirstFrameCallback,
    ) -> Result<Self, EngineError> {
        if let Some(project) = WebProject::load(&desc.scene_path)? {
            let mut scene = WebScene::open(project, desc, window, callback)?;
            scene.set_scaling_factor(scaling_factor)?;
            Ok(Self::Web(scene))
        } else {
            backend
                .open_scene(
                    desc,
                    window.metal_layer_ptr(),
                    scaling_mode,
                    scaling_factor,
                    resolution,
                    Some(callback),
                )
                .map(Self::Native)
        }
    }

    pub(super) fn is_web(&self) -> bool {
        matches!(self, Self::Web(_))
    }
    forward!(set_scaling_factor(factor: f64));
    forward!(set_target_fps(fps: u32));
    forward!(set_paused(paused: bool));
    forward!(set_mouse_position(x: f64, y: f64));
    forward!(set_mouse_button(button: u32, pressed: bool));
    forward!(set_mouse_entered(entered: bool));
    forward!(set_audio_volume(volume: AudioVolume));
    forward!(set_audio_muted(muted: bool));
    forward!(set_property_override(json: &str));
    forward!(reset_property_override());
    forward!(close());

    pub(super) fn set_scaling_mode(&mut self, mode: ScalingMode) -> Result<(), EngineError> {
        match self {
            Self::Native(scene) => scene.set_scaling_mode(mode),
            // HTML lays out responsively within its own viewport.
            Self::Web(_) => Ok(()),
        }
    }
    pub(super) fn set_audio_response_enabled(&mut self, enabled: bool) -> Result<(), EngineError> {
        match self {
            Self::Native(scene) => scene.set_audio_response_enabled(enabled),
            // Web audio-spectrum callbacks are not implemented by this backend.
            Self::Web(_) => Ok(()),
        }
    }
    pub(super) fn begin_surface_reconfigure(&mut self) -> Result<(), EngineError> {
        match self {
            Self::Native(scene) => scene.begin_surface_reconfigure(),
            Self::Web(_) => Err(EngineError::Render("WebKit has no Vulkan surface".into())),
        }
    }
    pub(super) fn finish_surface_reconfigure(
        &mut self,
        layer: *mut c_void,
        width: u32,
        height: u32,
        render_width: u32,
        render_height: u32,
        scale: f64,
    ) -> Result<(), EngineError> {
        match self {
            Self::Native(scene) => scene.finish_surface_reconfigure(
                layer,
                width,
                height,
                render_width,
                render_height,
                scale,
            ),
            Self::Web(_) => Err(EngineError::Render("WebKit has no Vulkan surface".into())),
        }
    }
}
