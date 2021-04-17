use std::time::Instant;
extern crate gl;

fn main() {
    let sdl_context = sdl2::init().expect("Cannot initialize sdl");
    let sdl_video = sdl_context.video().expect("Cannot initialize video");
    let window = sdl_video.window("CHIP-8", 800, 600)
        .opengl()
        .build()
        .expect("Cannot create windows");

    let _gl_context = window.gl_create_context().expect("Couldn't create GL context");
    gl::load_with(|s| sdl_video.gl_get_proc_address(s) as _);

    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);

    let mut imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);


    let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| sdl_video.gl_get_proc_address(s) as _);

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut last_frame = Instant::now();

    'running: loop {
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;

        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            if imgui_sdl2.ignore_event(&event) { continue; }

            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }


        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());

        let now = Instant::now();
        let delta = now - last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        last_frame = now;
        imgui.io_mut().delta_time = delta_s;

        let ui = imgui.frame();
        ui.show_demo_window(&mut true);

        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        imgui_sdl2.prepare_render(&ui, &window);
        renderer.render(ui);

        window.gl_swap_window();

        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}
