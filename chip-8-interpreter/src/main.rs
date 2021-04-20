extern crate gl;

use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

use chip_8::{cpu, display};
use chip_8::cpu::Cpu;

const FIXED_TIME: f64 = 1.0 / 10000.0;

fn main() {
    let filename = std::env::args()
        .nth(1)
        .expect("El primer argumento debe ser el archivo");
    let cycles_per_frame = std::env::args()
        .nth(2)
        .map_or(17, |s| i32::from_str(s.as_str()).unwrap_or(17));

    let file = std::fs::File::open(filename).expect("No se puede abrir el archivo");
    let mut cpu = cpu::Cpu::new(file).expect("No se pudo leer la memoria del archivo");

    // SDL Context creation
    let sdl_context = sdl2::init().expect("Cannot initialize sdl");
    let sdl_video = sdl_context.video().expect("Cannot initialize video");

    // SDL Window
    let window = sdl_video
        .window("CHIP-8", 800, 600)
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
            for data in buffer {
                *data = 0;
            }
        })
        .expect("No se pudo copiar");

    // let window = canvas.window();

    let mut current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();

    let mut accumulator = 0.0;
    let mut delta_time = 0.01;

    'running: loop {
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'running;
                }
                // 5 6 7 8
                // T Y U I
                // G H J K
                // V B N M
                Event::KeyDown {
                    keycode: Some(Keycode::Num5),
                    ..
                } => cpu.set_key(0, true),
                Event::KeyDown {
                    keycode: Some(Keycode::Num6),
                    ..
                } => cpu.set_key(1, true),
                Event::KeyDown {
                    keycode: Some(Keycode::Num7),
                    ..
                } => cpu.set_key(2, true),
                Event::KeyDown {
                    keycode: Some(Keycode::Num8),
                    ..
                } => cpu.set_key(3, true),

                Event::KeyDown {
                    keycode: Some(Keycode::T),
                    ..
                } => cpu.set_key(4, true),
                Event::KeyDown {
                    keycode: Some(Keycode::Y),
                    ..
                } => cpu.set_key(5, true),
                Event::KeyDown {
                    keycode: Some(Keycode::U),
                    ..
                } => cpu.set_key(6, true),
                Event::KeyDown {
                    keycode: Some(Keycode::I),
                    ..
                } => cpu.set_key(7, true),

                Event::KeyDown {
                    keycode: Some(Keycode::G),
                    ..
                } => cpu.set_key(8, true),
                Event::KeyDown {
                    keycode: Some(Keycode::H),
                    ..
                } => cpu.set_key(9, true),
                Event::KeyDown {
                    keycode: Some(Keycode::J),
                    ..
                } => cpu.set_key(10, true),
                Event::KeyDown {
                    keycode: Some(Keycode::K),
                    ..
                } => cpu.set_key(11, true),

                Event::KeyDown {
                    keycode: Some(Keycode::V),
                    ..
                } => cpu.set_key(12, true),
                Event::KeyDown {
                    keycode: Some(Keycode::B),
                    ..
                } => cpu.set_key(13, true),
                Event::KeyDown {
                    keycode: Some(Keycode::N),
                    ..
                } => cpu.set_key(14, true),
                Event::KeyDown {
                    keycode: Some(Keycode::M),
                    ..
                } => cpu.set_key(15, true),

                Event::KeyUp {
                    keycode: Some(Keycode::Num5),
                    ..
                } => cpu.set_key(0, false),
                Event::KeyUp {
                    keycode: Some(Keycode::Num6),
                    ..
                } => cpu.set_key(1, false),
                Event::KeyUp {
                    keycode: Some(Keycode::Num7),
                    ..
                } => cpu.set_key(2, false),
                Event::KeyUp {
                    keycode: Some(Keycode::Num8),
                    ..
                } => cpu.set_key(3, false),

                Event::KeyUp {
                    keycode: Some(Keycode::T),
                    ..
                } => cpu.set_key(4, false),
                Event::KeyUp {
                    keycode: Some(Keycode::Y),
                    ..
                } => cpu.set_key(5, false),
                Event::KeyUp {
                    keycode: Some(Keycode::U),
                    ..
                } => cpu.set_key(6, false),
                Event::KeyUp {
                    keycode: Some(Keycode::I),
                    ..
                } => cpu.set_key(7, false),

                Event::KeyUp {
                    keycode: Some(Keycode::G),
                    ..
                } => cpu.set_key(8, false),
                Event::KeyUp {
                    keycode: Some(Keycode::H),
                    ..
                } => cpu.set_key(9, false),
                Event::KeyUp {
                    keycode: Some(Keycode::J),
                    ..
                } => cpu.set_key(10, false),
                Event::KeyUp {
                    keycode: Some(Keycode::K),
                    ..
                } => cpu.set_key(11, false),

                Event::KeyUp {
                    keycode: Some(Keycode::V),
                    ..
                } => cpu.set_key(12, false),
                Event::KeyUp {
                    keycode: Some(Keycode::B),
                    ..
                } => cpu.set_key(13, false),
                Event::KeyUp {
                    keycode: Some(Keycode::N),
                    ..
                } => cpu.set_key(14, false),
                Event::KeyUp {
                    keycode: Some(Keycode::M),
                    ..
                } => cpu.set_key(15, false),

                _ => {}
            }
        }

        let new_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let mut frame_time = new_time - current_time;
        if frame_time > FIXED_TIME {
            frame_time = FIXED_TIME;
        }
        current_time = new_time;

        accumulator += frame_time;

        let mut counter = 0.0;

        for _ in 0..cycles_per_frame {
            cpu.next();
        }

        while accumulator > delta_time {
            let t = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            delta_time = t - current_time;

            accumulator -= delta_time;
            if counter < 0.16 {
                counter += delta_time;
            } else {
                counter = 0.0;
                cpu.decrease_timers();
            }
        }

        draw(&mut cpu, &mut canvas, &mut texture);
    }
}

fn draw(cpu: &mut Box<Cpu>, canvas: &mut Canvas<Window>, texture: &mut Texture) {
    canvas.clear();
    texture
        .with_lock(None, |buffer, _| {
            let video_buffer = cpu.get_display().get_video_mem();
            for (i, data) in video_buffer.iter().enumerate() {
                let draw_pixel = if *data == 0 { 0 } else { 255 };
                buffer[i * 3] = draw_pixel;
                buffer[i * 3 + 1] = draw_pixel;
                buffer[i * 3 + 2] = draw_pixel;
            }
        })
        .expect("No se pudo copiar");

    canvas
        .copy(&texture, None, None)
        .expect("No se pudo copiar al render buffer");
    canvas.present();
}
