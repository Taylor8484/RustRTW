//! A screenshot for checking the renderer without watching it:
//! `WL_SCREENSHOT=out.png[,frames]` saves the window after `frames` frames
//! (default 300) and quits. With `WL_HEADLESS=1` as well there is no window:
//! the camera draws into a 1024x768 image (see `main.rs`), for machines
//! that cannot present one (WSLg, CI); `WL_HEADLESS_SIZE=<w>x<h>` picks
//! another size.

use std::path::PathBuf;

use bevy::app::AppExit;
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};

/// The headless render target's size: `WL_HEADLESS_SIZE`, else 1024x768.
pub fn headless_size() -> (u32, u32) {
    std::env::var("WL_HEADLESS_SIZE")
        .ok()
        .and_then(|v| {
            let (w, h) = v.split_once('x')?;
            Some((w.trim().parse().ok()?, h.trim().parse().ok()?))
        })
        .filter(|&(w, h)| w > 0 && h > 0)
        .unwrap_or((1024, 768))
}

/// Whether `WL_HEADLESS` asks for rendering without a window.
pub fn headless() -> bool {
    std::env::var_os("WL_HEADLESS").is_some_and(|v| !v.is_empty() && v != "0")
}

/// The image the camera draws into when headless.
#[derive(Resource, Clone)]
pub struct HeadlessTarget(pub Handle<Image>);

impl HeadlessTarget {
    pub fn render_target(&self) -> RenderTarget {
        RenderTarget::Image(self.0.clone().into())
    }
}

#[derive(Resource)]
struct Shot {
    path: PathBuf,
    frames: u32,
    seen: u32,
}

pub struct ShotPlugin;

impl Plugin for ShotPlugin {
    fn build(&self, app: &mut App) {
        let Some(spec) = std::env::var_os("WL_SCREENSHOT") else {
            return;
        };
        let spec = spec.to_string_lossy().into_owned();
        let (path, frames) = match spec.rsplit_once(',') {
            Some((p, n)) if n.parse::<u32>().is_ok() => (p.to_owned(), n.parse().unwrap()),
            _ => (spec, 300),
        };
        app.insert_resource(Shot {
            path: PathBuf::from(path),
            frames,
            seen: 0,
        })
        .add_systems(Update, take);
        if headless() {
            app.add_systems(PreStartup, make_target);
        }
    }
}

fn make_target(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let (w, h) = headless_size();
    let image = Image::new_target_texture(w, h, TextureFormat::Rgba8UnormSrgb, None);
    commands.insert_resource(HeadlessTarget(images.add(image)));
}

fn take(mut commands: Commands, mut shot: ResMut<Shot>, target: Option<Res<HeadlessTarget>>) {
    shot.seen += 1;
    if shot.seen != shot.frames {
        return;
    }
    let path = shot.path.clone();
    let screenshot = match target {
        Some(t) => Screenshot::image(t.0.clone()),
        None => Screenshot::primary_window(),
    };
    commands.spawn(screenshot).observe(
        move |captured: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
            match captured.image.clone().try_into_dynamic() {
                Ok(img) => match img.to_rgb8().save(&path) {
                    Ok(()) => info!("saved {}", path.display()),
                    Err(e) => error!("{}: {e}", path.display()),
                },
                Err(e) => error!("screenshot: {e:?}"),
            }
            exit.write(AppExit::Success);
        },
    );
}
