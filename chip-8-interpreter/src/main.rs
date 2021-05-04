#![windows_subsystem = "windows"]

extern crate gl;

use std::io::Read;

use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use serde::Deserialize;

use chip_8::{cpu, display};
use chip_8::cpu::Cpu;

mod audio;

#[derive(Deserialize)]
struct ColorConfig {
    back: [u8; 3],
    front: [u8; 3],
}

#[derive(Deserialize)]
struct Config {
    color: ColorConfig,
    executable: String,
    cycles_per_second: i32,
}

fn main() {
    let config = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.toml".into());


    let mut interpreter_config = String::new();
    let _ = std::fs::File::open(config.as_str()).unwrap().read_to_string(&mut interpreter_config);


    let config = toml::from_str::<Config>(interpreter_config.as_str())
        .expect("No se puede cargar el archivo de configuración");

    let file = std::fs::File::open(config.executable.as_str()).expect("No se puede abrir el archivo");
    let mut cpu = cpu::Cpu::new(file).expect("No se pudo leer la memoria del archivo");

    // SDL Context creation
    let sdl_context = sdl2::init().expect("Cannot initialize sdl");
    let sdl_video = sdl_context.video().expect("Cannot initialize video");

    let audio_device = audio::initialize(&sdl_context).expect("No se puede cargar el audio");
    let mut playing = false;

    // SDL Window
    let window = sdl_video
        .window("CHIP-8", 640, 320)
        .opengl()
        .resizable()
        .build()
        .expect("Cannot create windows");

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .expect("No se puede obtener un contexto gráfico");

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::RGB24,
            display::WIDTH as u32,
            display::HEIGHT as u32,
        )
        .expect("No se puede crear la textura");

    texture
        .with_lock(None, |buffer, _| {
            for i in (0..buffer.len()).step_by(3) {
                let color = &config.color.back;
                buffer[i] = color[0];
                buffer[i + 1] = color[1];
                buffer[i + 2] = color[2];
            }
            for data in buffer {
                *data = 0;
            }
        })
        .expect("No se pudo copiar");

    'running: loop {
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'running;
                }
                // 1 2 3 C
                // 4 5 6 D
                // 7 8 9 E
                // A 0 B F
                // Mapped to
                // 1 2 3 4
                // q w e r
                // a s d f
                // z x c v
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => cpu.set_key(1, true),
                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => cpu.set_key(2, true),
                Event::KeyDown {
                    keycode: Some(Keycode::Num3),
                    ..
                } => cpu.set_key(3, true),
                Event::KeyDown {
                    keycode: Some(Keycode::Num4),
                    ..
                } => cpu.set_key(0xC, true),

                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => cpu.set_key(0x4, true),
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => cpu.set_key(0x5, true),
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    ..
                } => cpu.set_key(0x6, true),
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => cpu.set_key(0xD, true),

                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => cpu.set_key(0x7, true),
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => cpu.set_key(0x8, true),
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => cpu.set_key(0x9, true),
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => cpu.set_key(0xE, true),

                Event::KeyDown {
                    keycode: Some(Keycode::Z),
                    ..
                } => cpu.set_key(0xA, true),
                Event::KeyDown {
                    keycode: Some(Keycode::X),
                    ..
                } => cpu.set_key(0x0, true),
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => cpu.set_key(0xB, true),
                Event::KeyDown {
                    keycode: Some(Keycode::V),
                    ..
                } => cpu.set_key(0xF, true),

                Event::KeyUp {
                    keycode: Some(Keycode::Num1),
                    ..
                } => cpu.set_key(1, false),
                Event::KeyUp {
                    keycode: Some(Keycode::Num2),
                    ..
                } => cpu.set_key(2, false),
                Event::KeyUp {
                    keycode: Some(Keycode::Num3),
                    ..
                } => cpu.set_key(3, false),
                Event::KeyUp {
                    keycode: Some(Keycode::Num4),
                    ..
                } => cpu.set_key(0xC, false),

                Event::KeyUp {
                    keycode: Some(Keycode::Q),
                    ..
                } => cpu.set_key(0x4, false),
                Event::KeyUp {
                    keycode: Some(Keycode::W),
                    ..
                } => cpu.set_key(0x5, false),
                Event::KeyUp {
                    keycode: Some(Keycode::E),
                    ..
                } => cpu.set_key(0x6, false),
                Event::KeyUp {
                    keycode: Some(Keycode::R),
                    ..
                } => cpu.set_key(0xD, false),

                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => cpu.set_key(0x7, false),
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => cpu.set_key(0x8, false),
                Event::KeyUp {
                    keycode: Some(Keycode::D),
                    ..
                } => cpu.set_key(0x9, false),
                Event::KeyUp {
                    keycode: Some(Keycode::F),
                    ..
                } => cpu.set_key(0xE, false),

                Event::KeyUp {
                    keycode: Some(Keycode::Z),
                    ..
                } => cpu.set_key(0xA, false),
                Event::KeyUp {
                    keycode: Some(Keycode::X),
                    ..
                } => cpu.set_key(0x0, false),
                Event::KeyUp {
                    keycode: Some(Keycode::C),
                    ..
                } => cpu.set_key(0xB, false),
                Event::KeyUp {
                    keycode: Some(Keycode::V),
                    ..
                } => cpu.set_key(0xF, false),

                _ => {}
            }
        }

        for _ in 0..config.cycles_per_second {
            cpu.next();
        }

        canvas.clear();
        draw(&mut cpu, &mut canvas, &mut texture, &config);
        canvas.present();
        cpu.decrease_timers();
        if cpu.as_ref().should_play_sound() && !playing {
            audio_device.resume();
            playing = true;
        }
        if !cpu.as_ref().should_play_sound() && playing {
            audio_device.pause();
            playing = false;
        }
    }
}

fn draw(cpu: &mut Box<Cpu>, canvas: &mut Canvas<Window>, texture: &mut Texture, config: &Config) {
    texture
        .with_lock(None, |buffer, _| {
            let video_buffer = cpu.get_display().get_video_mem();
            let front_color = &config.color.front;
            let back_color = &config.color.back;
            for (i, data) in video_buffer.iter().enumerate() {
                let draw_pixel = *data != 0;

                buffer[i * 3] = if draw_pixel { front_color[0] } else { back_color[0] };
                buffer[i * 3 + 1] = if draw_pixel { front_color[1] } else { back_color[1] };
                buffer[i * 3 + 2] = if draw_pixel { front_color[2] } else { back_color[2] };
            }
        })
        .expect("No se pudo copiar");

    canvas
        .copy(&texture, None, None)
        .expect("No se pudo copiar al render buffer");
}
