use std::{
    ffi::{CStr, CString, c_char, c_void},
    path::{Path, PathBuf},
    sync::{Mutex, mpsc},
    time::Duration,
};

use serde_json::{Value, json};

use crate::{
    EngineError, WallpaperWindow,
    media::audio::AudioVolume,
    owe::backend::FirstFrameCallback,
    project::{ProjectManifest, SceneDesc, WallpaperProjectType, validate_relative_normal_path},
    window::MainThread,
};

unsafe extern "C" {
    fn we_web_open(
        parent: *mut c_void,
        entry: *const c_char,
        root: *const c_char,
        script: *const c_char,
        callback: extern "C" fn(*mut c_void, i32, *const c_char),
        context: *mut c_void,
    ) -> *mut c_void;
    fn we_web_evaluate(raw: *mut c_void, script: *const c_char);
    fn we_web_set_zoom(raw: *mut c_void, zoom: f64);
    fn we_web_close(raw: *mut c_void);
}

#[derive(Debug)]
pub(crate) struct WebProject {
    entry: PathBuf,
    root: PathBuf,
    properties: Value,
}

impl WebProject {
    pub(crate) fn load(path: impl AsRef<Path>) -> Result<Option<Self>, EngineError> {
        let path = path.as_ref();
        if path.file_name().and_then(|name| name.to_str()) != Some("project.json") {
            return Ok(None);
        }
        let manifest = ProjectManifest::load(path)?;
        if manifest.project_type() != WallpaperProjectType::Web {
            return Ok(None);
        }
        let entry = Path::new(manifest.file());
        validate_relative_normal_path(entry, "web project entry")?;
        let root = path
            .parent()
            .unwrap_or(Path::new("."))
            .canonicalize()
            .map_err(|error| {
                EngineError::InvalidInput(format!("cannot resolve web project: {error}"))
            })?;
        let entry = root.join(entry).canonicalize().map_err(|error| {
            EngineError::InvalidInput(format!("cannot read web entry: {error}"))
        })?;
        if !entry.starts_with(&root) || !entry.is_file() {
            return Err(EngineError::InvalidInput(
                "web entry must be a file inside its project directory".into(),
            ));
        }
        let document: Value = serde_json::from_slice(
            &std::fs::read(path).map_err(|error| EngineError::InvalidInput(error.to_string()))?,
        )
        .map_err(|error| EngineError::InvalidInput(error.to_string()))?;
        let properties = document
            .get("general")
            .and_then(|value| value.get("properties"))
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        Ok(Some(Self {
            entry,
            root,
            properties: Value::Object(properties),
        }))
    }

    fn merged_properties(&self, overrides: Option<&str>) -> Result<Value, EngineError> {
        merge_properties(&self.properties, overrides)
    }
}

fn merge_properties(defaults: &Value, overrides: Option<&str>) -> Result<Value, EngineError> {
    let mut properties = defaults.clone();
    if let Some(overrides) = overrides {
        let values: Value = serde_json::from_str(overrides).map_err(|error| {
            EngineError::InvalidInput(format!("invalid web properties: {error}"))
        })?;
        let values = values
            .as_object()
            .ok_or_else(|| EngineError::InvalidInput("web properties must be an object".into()))?;
        for (name, value) in values {
            let property = properties
                .as_object_mut()
                .expect("defaults are an object")
                .entry(name)
                .or_insert_with(|| json!({}));
            if !property.is_object() {
                *property = json!({});
            }
            property["value"] = value.get("value").unwrap_or(value).clone();
        }
    }
    Ok(properties)
}

struct Callbacks {
    loaded: Mutex<Option<mpsc::SyncSender<Result<(), String>>>>,
    first_frame: FirstFrameCallback,
}

extern "C" fn web_event(context: *mut c_void, kind: i32, message: *const c_char) {
    // SAFETY: Native callbacks are confined to the main thread. The Box stays
    // alive until synchronous we_web_close has detached every callback source.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let callbacks = &*context.cast::<Callbacks>();
        let message = CStr::from_ptr(message).to_string_lossy();
        if kind < 0 {
            log::error!("[web wallpaper] {message}");
        }
        if kind <= 0 {
            if let Some(sender) = callbacks
                .loaded
                .lock()
                .ok()
                .and_then(|mut sender| sender.take())
            {
                let _ = sender.send(if kind == 0 {
                    Ok(())
                } else {
                    Err(message.into_owned())
                });
            }
        } else if kind == 1 {
            (callbacks.first_frame)();
        }
    }));
}

pub(crate) struct WebScene {
    // Opaque main-thread-only Objective-C token, never dereferenced in Rust.
    raw: Option<usize>,
    _callbacks: Box<Callbacks>,
    project: WebProject,
    config: Value,
    mouse_position: Option<(f64, f64)>,
}

impl WebScene {
    pub(crate) fn open(
        project: WebProject,
        desc: &SceneDesc,
        window: &WallpaperWindow,
        callback: FirstFrameCallback,
    ) -> Result<Self, EngineError> {
        if objc2_foundation::NSThread::isMainThread_class() {
            return Err(EngineError::Platform(
                "web loading must run outside the AppKit main thread".into(),
            ));
        }
        let config = json!({"properties": project.merged_properties(desc.property_override_json.as_deref())?,
            "fps":desc.fps.max(1), "paused":desc.paused, "volume":f32::from(desc.audio_volume), "muted":desc.audio_muted});
        let script = CString::new(
            include_str!("host.js").replace("__WALLPAPER_CONFIG__", &config.to_string()),
        )
        .map_err(|error| EngineError::InvalidInput(error.to_string()))?;
        let entry = CString::new(project.entry.to_string_lossy().as_bytes())
            .map_err(|error| EngineError::InvalidInput(error.to_string()))?;
        let root = CString::new(project.root.to_string_lossy().as_bytes())
            .map_err(|error| EngineError::InvalidInput(error.to_string()))?;
        let (sender, receiver) = mpsc::sync_channel(1);
        let mut callbacks = Box::new(Callbacks {
            loaded: Mutex::new(Some(sender)),
            first_frame: callback,
        });
        let context = (&raw mut *callbacks) as usize;
        let parent = window.content_view_ptr() as usize;
        // SAFETY: The window stays alive for this runtime; native code attaches
        // and retains its own view. All Objective-C calls run on the main
        // thread.
        let raw = MainThread::dispatch(move || unsafe {
            we_web_open(
                parent as *mut c_void,
                entry.as_ptr(),
                root.as_ptr(),
                script.as_ptr(),
                web_event,
                context as *mut c_void,
            ) as usize
        });
        if raw == 0 {
            return Err(EngineError::Render("cannot create WebKit wallpaper".into()));
        }
        let mut scene = Self {
            raw: Some(raw),
            _callbacks: callbacks,
            project,
            config,
            mouse_position: None,
        };
        match receiver.recv_timeout(Duration::from_secs(20)) {
            Ok(Ok(())) => {
                log::info!(
                    "[web wallpaper] loaded {} on display {} ({}x{}, scale={})",
                    scene.project.entry.display(),
                    desc.display.display_id,
                    desc.display.width,
                    desc.display.height,
                    desc.display.scale_factor
                );
                Ok(scene)
            }
            result => {
                scene.close()?;
                Err(EngineError::Render(match result {
                    Ok(Err(error)) => format!("web wallpaper failed to load: {error}"),
                    _ => "web wallpaper loading timed out".into(),
                }))
            }
        }
    }

    fn evaluate(&self, script: String) -> Result<(), EngineError> {
        let raw = self
            .raw
            .ok_or_else(|| EngineError::Render("web wallpaper is closed".into()))?;
        let script =
            CString::new(script).map_err(|error| EngineError::InvalidInput(error.to_string()))?;
        // SAFETY: The owned token cannot be closed during this synchronous
        // call.
        MainThread::dispatch(move || unsafe {
            we_web_evaluate(raw as *mut c_void, script.as_ptr())
        });
        Ok(())
    }

    fn apply(&self) -> Result<(), EngineError> {
        self.evaluate(format!("window.__wallpaperHost.apply({})", self.config))
    }

    pub(crate) fn set_target_fps(&mut self, fps: u32) -> Result<(), EngineError> {
        self.config["fps"] = json!(fps.max(1));
        self.apply()
    }
    pub(crate) fn set_paused(&mut self, paused: bool) -> Result<(), EngineError> {
        self.config["paused"] = json!(paused);
        self.apply()
    }
    pub(crate) fn set_audio_volume(&mut self, volume: AudioVolume) -> Result<(), EngineError> {
        self.config["volume"] = json!(f32::from(volume));
        self.apply()
    }
    pub(crate) fn set_audio_muted(&mut self, muted: bool) -> Result<(), EngineError> {
        self.config["muted"] = json!(muted);
        self.apply()
    }
    pub(crate) fn set_property_override(&mut self, overrides: &str) -> Result<(), EngineError> {
        self.config["properties"] = self.project.merged_properties(Some(overrides))?;
        self.apply()
    }
    pub(crate) fn reset_property_override(&mut self) -> Result<(), EngineError> {
        self.config["properties"] = self.project.properties.clone();
        self.apply()
    }
    pub(crate) fn set_scaling_factor(&mut self, factor: f64) -> Result<(), EngineError> {
        let raw = self
            .raw
            .ok_or_else(|| EngineError::Render("web wallpaper is closed".into()))?;
        // SAFETY: Opaque token is used only on the main thread while owned.
        MainThread::dispatch(move || unsafe { we_web_set_zoom(raw as *mut c_void, factor) });
        Ok(())
    }
    pub(crate) fn set_mouse_position(&mut self, x: f64, y: f64) -> Result<(), EngineError> {
        if self.mouse_position == Some((x, y)) {
            return Ok(());
        }
        self.mouse_position = Some((x, y));
        self.evaluate(format!(
            "window.__wallpaperHost.input({})",
            json!({"type":"mousemove","x":x,"y":y})
        ))
    }
    pub(crate) fn set_mouse_button(
        &mut self,
        button: u32,
        pressed: bool,
    ) -> Result<(), EngineError> {
        self.evaluate(format!(
            "window.__wallpaperHost.input({})",
            json!({"type":if pressed {"mousedown"} else {"mouseup"},"button":button})
        ))
    }
    pub(crate) fn set_mouse_entered(&mut self, entered: bool) -> Result<(), EngineError> {
        self.evaluate(format!(
            "window.__wallpaperHost.input({})",
            json!({"type":if entered {"mouseenter"} else {"mouseleave"}})
        ))
    }
    pub(crate) fn close(&mut self) -> Result<(), EngineError> {
        if let Some(raw) = self.raw.take() {
            // SAFETY: Native close detaches delegate/message handlers before
            // the callback Box is dropped, and consumes the retained token
            // once.
            MainThread::dispatch(move || unsafe { we_web_close(raw as *mut c_void) });
        }
        Ok(())
    }
}

impl Drop for WebScene {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_properties_merge_preserves_metadata_and_default_values() {
        let defaults =
            json!({"mode":{"type":"combo","value":"single"},"zoom":{"type":"slider","value":100}});
        let result =
            merge_properties(&defaults, Some(r#"{"zoom":175,"mode":{"value":"pair"}}"#)).unwrap();
        assert_eq!(result["zoom"], json!({"type":"slider","value":175}));
        assert_eq!(result["mode"]["value"], "pair");
        assert_eq!(merge_properties(&defaults, None).unwrap(), defaults);
        assert!(merge_properties(&defaults, Some("[]")).is_err());
    }

    #[test]
    fn web_entry_must_exist_and_remain_inside_project() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = dir.path().join("project.json");
        let write = |entry: &str| {
            std::fs::write(&manifest, json!({"type":"Web","file":entry}).to_string()).unwrap()
        };
        write("index.html");
        assert!(WebProject::load(&manifest).is_err());
        std::fs::write(dir.path().join("index.html"), "<canvas></canvas>").unwrap();
        assert!(WebProject::load(&manifest).unwrap().is_some());
        for entry in ["../index.html", "/etc/passwd", "", "..\\index.html"] {
            write(entry);
            assert!(WebProject::load(&manifest).is_err());
        }
        let outside = tempfile::NamedTempFile::new().unwrap();
        std::os::unix::fs::symlink(outside.path(), dir.path().join("escape.html")).unwrap();
        write("escape.html");
        assert!(WebProject::load(&manifest).is_err());
    }
}
