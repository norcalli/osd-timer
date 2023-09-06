use std::collections::HashSet;
use std::time::{Duration, Instant};

use chrono::Local;
use macroquad::miniquad::conf::Platform;
use macroquad::prelude::*;

use macroquad_grid::Grid;
use static_init::dynamic;

pub mod fmt;

#[macro_export]
macro_rules! id {
    ($($s:expr),*) => {{
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut s = DefaultHasher::new();
        concat!(file!(), line!(), column!()).hash(&mut s);
        $(($s).hash(&mut s);)*
        s.finish()
    }};
}

fn window_conf() -> Conf {
    Conf {
        window_title: "OSD Timer".to_owned(),
        fullscreen: false,
        high_dpi: true,

        window_width: OPTIONS.window_width as i32,
        window_height: OPTIONS.window_height as i32,
        window_resizable: false,

        platform: Platform {
            // linux_backend: LinuxBackend::WaylandWithX11Fallback,
            // wayland_use_fallback_decorations: false,
            // framebuffer_alpha: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn default_time_format() -> String {
    "%T%.3f".to_string()
}

pub fn default_window_width() -> usize {
    800
}
pub fn default_window_height() -> usize {
    200
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Options {
    #[serde(default = "default_time_format")]
    pub time_format: String,
    #[serde(default = "default_window_width")]
    pub window_width: usize,
    #[serde(default = "default_window_height")]
    pub window_height: usize,
    #[serde(default)]
    pub deadline: Option<chrono::DateTime<Local>>,
}

#[dynamic]
static OPTIONS: Options = serde_qs::from_str(&std::env::args().nth(1).unwrap_or_default()).unwrap();

#[macroquad::main(window_conf)]
async fn main() {
    let mut arena = bumpalo::Bump::new();
    let mut grid = Grid::new(
        screen_width(),
        screen_height(),
        2 + OPTIONS.deadline.is_some() as usize,
        1,
        5.0,
    );
    let mut timer_total = Duration::from_secs(0);
    let mut timer_start: Option<Instant> = None;
    // let min_size = grid.dimensions();
    #[derive(Hash, Eq, PartialEq)]
    enum Commands {
        ToggleTimer,
        IncrementTimer,
        DecrementTimer,
        PauseTimer,
        StartTimer,
        ClearTimer,
        Quit,
    }
    let mut commands = HashSet::new();
    'outer: loop {
        arena.reset();
        clear_background(WHITE);
        set_default_camera();
        let ts = chrono::Local::now().format(&OPTIONS.time_format);
        grid.set_cell_text(0, 0, Some(ts));
        (grid.width, grid.height) = (screen_width(), screen_height());
        if grid.mouse_hovered_cell() == Some((0, 1)) {
            if is_mouse_button_pressed(MouseButton::Left) {
                commands.insert(Commands::ToggleTimer);
            }
            if is_mouse_button_pressed(MouseButton::Right) {
                commands.insert(Commands::ClearTimer);
            }
        }
        match get_char_pressed() {
            Some('=') | Some('+') => {
                commands.insert(Commands::IncrementTimer);
            }
            Some('-') => {
                commands.insert(Commands::DecrementTimer);
            }
            Some('q') => {
                commands.insert(Commands::Quit);
            }
            Some(' ') => {
                commands.insert(Commands::PauseTimer);
            }
            Some('r') => {
                commands.insert(Commands::ClearTimer);
            }
            Some('p') => {
                commands.insert(Commands::StartTimer);
            }
            _ => (),
        }
        if timer_start.is_some() {
            grid.color_cell(0, 1, GREEN);
        } else {
            grid.color_cell(0, 1, RED);
        }
        loop {
            let Some(command) = commands.drain().next() else {
                break;
            };
            match command {
                Commands::ToggleTimer => {
                    timer_start = match timer_start {
                        Some(start) => {
                            timer_total += start.elapsed();
                            None
                        }
                        None => Some(Instant::now()),
                    };
                }
                Commands::IncrementTimer => {
                    timer_total += Duration::from_secs(1);
                }
                Commands::DecrementTimer => {
                    timer_total -= timer_total.min(Duration::from_secs(1));
                }
                Commands::StartTimer => {
                    if timer_start.is_none() {
                        commands.insert(Commands::ToggleTimer);
                    }
                }
                Commands::PauseTimer => {
                    if timer_start.is_some() {
                        commands.insert(Commands::ToggleTimer);
                    }
                }
                Commands::ClearTimer => {
                    timer_total = Duration::from_secs(0);
                    timer_start = None;
                }
                Commands::Quit => {
                    break 'outer;
                }
            }
        }
        // let new_size = if is_key_pressed(KeyCode::KpAdd) {
        //     let mut dim = grid.dimensions();
        //     dim.cols += 1;
        //     dim
        // } else is_key_pressed(KeyCode::KpAdd){
        //     let mut dim = grid.dimensions();
        //     dim.cols += 1;
        //     dim
        // }.max(min_size);
        let current_elapsed = timer_total + timer_start.map(|ts| ts.elapsed()).unwrap_or_default();
        {
            let fmt = fmt::FmtFn(|f| {
                let mut secs = current_elapsed.as_secs();
                let hours = secs / 3600;
                secs -= hours * 3600;
                let mins = secs / 60;
                secs -= mins * 60;
                if hours > 0 {
                    write!(f, " {hours}h")?;
                }
                if mins > 0 {
                    write!(f, " {mins}m")?;
                }
                write!(f, " {secs}.{:03}s", current_elapsed.as_millis() % 1000)
            });
            grid.set_cell_text(0, 1, Some(bumpalo::format!(in &arena, "{}", fmt)));
        }
        if let Some(deadline) = OPTIONS.deadline {
            let duration = deadline
                .signed_duration_since(Local::now())
                .to_std()
                .unwrap_or_default();
            let fmt = fmt::FmtFn(|f| {
                let mut secs = duration.as_secs();
                let hours = secs / 3600;
                secs -= hours * 3600;
                let mins = secs / 60;
                secs -= mins * 60;
                if hours > 0 {
                    write!(f, " {hours}h")?;
                }
                if mins > 0 {
                    write!(f, " {mins}m")?;
                }
                write!(f, " {secs}s")
            });
            grid.set_cell_text(0, 2, Some(bumpalo::format!(in &arena, "{}", fmt)));
        }
        grid.draw();
        // draw_text_ex(
        //     &s,
        //     10.0,
        //     10.0,
        //     TextParams {
        //         // font: None,
        //         // font_size: 32,
        //         color: BLACK,
        //         ..TextParams::default()
        //     },
        // );

        next_frame().await
    }
}
