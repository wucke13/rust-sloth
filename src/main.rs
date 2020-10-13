use clap::Clap;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, terminal,
};
use std::error::Error;
use std::f32;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

use nalgebra::Rotation3;

pub mod context;
pub use context::*;

pub mod geometry;
pub use geometry::*;

pub mod rasterizer;
pub use rasterizer::*;

pub mod inputs;

fn main() -> Result<(), Box<dyn Error>> {
    let opts = inputs::Opts::parse(); // Read command line arguments

    let fps_cap = 500.0;
    let target_frame_time = Duration::from_secs_f64(1.0 / fps_cap);

    let mesh_queue: Vec<SimpleMesh> = opts.match_meshes()?; // A list of meshes to render
    let mut turntable = opts.match_turntable()?;
    let mut stdout = stdout();
    let no_color = opts.color_less;
    let mut webify = false;
    let mut webify_frame_count = 0;
    let mut webify_todo_frames = 0;

    let image_mode = match opts.subcmd {
        inputs::SubCommand::Image { .. } => true,
        _ => false,
    };

    // The context holds the frame+z buffer, and the width and height
    let mut context: Context = Context::blank(image_mode);

    if context.image {
        /*
        if let Some(matches) = image_mode {
            opts.match_dimensions(&mut context)?;
            turntable = opts.match_turntable()?;
            if let Some(animation_frames) = matches.value_of("frame count") {
                webify_todo_frames = animation_frames.parse()?;
                webify = true;
            }
        }*/
    } else {
        crossterm::terminal::enable_raw_mode()?;
        execute!(stdout, cursor::Hide, terminal::EnterAlternateScreen)?;
    }
    let size: (u16, u16) = (0, 0); // This is the terminal size, it's used to check when a new context must be made

    if webify {
        println!("let frames = [");
        turntable.3 = (2.0 * f32::consts::PI) * (1.0 / webify_todo_frames as f32);
    }
    let mut last_time; // Used in the variable time step
    loop {
        last_time = Instant::now();
        if !context.image {
            if event::poll(target_frame_time - last_time.elapsed())? {
                if let Event::Key(KeyEvent { code, modifiers }) = event::read()? {
                    if code == KeyCode::Char('q')
                        || (code == KeyCode::Char('c') && (modifiers == KeyModifiers::CONTROL))
                    {
                        execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
                        crossterm::terminal::disable_raw_mode()?;
                        break;
                    }
                }
            }
        }

        let rot =
            Rotation3::from_euler_angles(turntable.0, turntable.1, turntable.2).to_homogeneous();
        context.update(size, &mesh_queue)?; // This checks for if there needs to be a context update
        context.clear(); // This clears the z and frame buffer
        for mesh in &mesh_queue {
            // Render all in mesh queue
            draw_mesh(&mut context, &mesh, rot, default_shader); // Draw all meshes
        }

        if webify {
            println!("`");
        }

        context.flush(!no_color, webify)?; // This prints all framebuffer info

        let dt = Instant::now().duration_since(last_time).as_nanos() as f32 / 1_000_000_000.0;
        turntable.1 += if webify {
            turntable.3
        } else {
            (turntable.3 * dt) as f32
        };

        if webify {
            if turntable.1 > 9.42477 || webify_todo_frames - 1 == webify_frame_count {
                println!("`];");
                break;
            } else {
                println!("`,");
            }
            webify_frame_count += 1;
        }

        if context.image && !webify {
            break;
        }
    }

    Ok(())
}
