//! Pictures of parts of the shell without a compositor's surfaces, for looking at them while
//! working on them:
//!
//! `shoji-bar-4 preview <part> <out.png> [--width W] [--height H] [--scale S] [--light]
//!  [--pointer X,Y] [--click X,Y] [--ms N]`
//!
//! The pointer and clicks are in logical pixels; `--ms` says how long animations run before the
//! picture is taken.

use anti_webui::app::headless::write_png;
use anti_webui::core::Ui;
use anti_webui::core::event::{Modifiers, PointerButton, WindowEvent};
use anti_webui::core::geometry::{PointF, Scale, SizeI};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use anti_webui::wgpu::{Gpu, Renderer};
use std::time::{Duration, Instant};

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let part = args.first().cloned().unwrap_or_else(|| "bar".into());
    let out = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| format!("{part}.png"));
    let opt = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let point = |s: String| -> Option<(f32, f32)> {
        let (x, y) = s.split_once(',')?;
        Some((x.parse().ok()?, y.parse().ok()?))
    };
    let scale: f64 = opt("--scale").and_then(|s| s.parse().ok()).unwrap_or(1.8);
    let (default_w, default_h) = match part.as_str() {
        "bar" => (1600.0, 56.0),
        "dock" => (900.0, 150.0),
        "launcher" | "quick-settings" | "calendar" => (1600.0, 1000.0),
        _ => (800.0, 600.0),
    };
    let lw: f32 = opt("--width")
        .and_then(|s| s.parse().ok())
        .unwrap_or(default_w);
    let lh: f32 = opt("--height")
        .and_then(|s| s.parse().ok())
        .unwrap_or(default_h);
    let (w, h) = (
        (lw as f64 * scale).round() as u32,
        (lh as f64 * scale).round() as u32,
    );
    let light = args.iter().any(|a| a == "--light");
    let pointer = opt("--pointer").and_then(point);
    let clicks: Vec<(f32, f32)> = args
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--click")
        .filter_map(|(i, _)| args.get(i + 1).cloned().and_then(point))
        .collect();
    let ms: u64 = opt("--ms").and_then(|s| s.parse().ok()).unwrap_or(1200);

    let gpu = Gpu::headless()?;
    let mut renderer = Renderer::new(
        &gpu.device,
        anti_webui::wgpu::wgpu::TextureFormat::Rgba8Unorm,
    );
    let output = outputs().into_iter().next();
    let mut ui = Ui::new(move || {
        if light {
            crate::settings::settings().update(|s| s.mode = crate::settings::ThemeMode::Light);
        }
        crate::theme::install();
        let backdrop = div()
            .size_full()
            .bg(rgb(if light { 0xc8d0d8 } else { 0x30343a }));
        let Some(output) = output else {
            return backdrop.child("no output");
        };
        if let Some(panel) = crate::panels::Panel::from_name(&part) {
            crate::panels::open(&output.name, panel);
        }
        let view = match part.as_str() {
            "bar" => crate::bar::view(output),
            "notifications" => match crate::notify::server() {
                Some(server) => crate::notify::popups(server),
                None => div().child("not the notification daemon"),
            },
            "dock" => crate::dock::view(output, signal(InputRegion::Full)),
            "launcher" | "quick-settings" | "calendar" => {
                let panel = crate::panels::Panel::from_name(&part).unwrap();
                crate::panels::view(output, panel)
            }
            _ => div().child("unknown part"),
        };
        backdrop
            .relative()
            .child(div().absolute().inset(0.0).flex().flex_col().child(view))
    });
    let to_physical = |(x, y): (f32, f32)| PointF::new(x * scale as f32, y * scale as f32);
    ui.handle_event(WindowEvent::Resized {
        size: SizeI::new(w as i32, h as i32),
        scale: Scale::from_f64(scale),
    });
    ui.handle_event(WindowEvent::Focused(true));
    ui.update();
    let mut now = Instant::now();
    let mut step = |ui: &mut Ui, frames: u64| {
        for _ in 0..frames {
            // In real time, so that what the services send arrives on the way.
            std::thread::sleep(Duration::from_millis(16));
            now += Duration::from_millis(16);
            ui.tick(now);
            ui.animate(now);
            ui.update();
            ui.paint(now);
        }
    };
    // Lets the services and pictures arrive.
    let _ = ui.wait_for_images(Duration::from_secs(5));
    std::thread::sleep(Duration::from_millis(600));
    step(&mut ui, 30);
    let m = Modifiers::default();
    for c in clicks {
        let at = to_physical(c);
        ui.handle_event(WindowEvent::PointerMoved {
            position: at,
            modifiers: m,
        });
        ui.handle_event(WindowEvent::PointerButton {
            position: at,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: m,
            time: 0,
        });
        ui.handle_event(WindowEvent::PointerButton {
            position: at,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: m,
            time: 10,
        });
        step(&mut ui, 20);
    }
    if let Some(p) = pointer {
        ui.handle_event(WindowEvent::PointerMoved {
            position: to_physical(p),
            modifiers: m,
        });
    }
    let _ = ui.wait_for_images(Duration::from_secs(5));
    step(&mut ui, ms / 16);
    ui.update();
    let scene = ui.paint(now);
    let pixels = gpu.render_to_rgba(&mut renderer, &scene);
    write_png(&out, w, h, &pixels)?;
    Ok(())
}
