mod cpu;
mod mem;

#[macroquad::main(window_config)]
async fn main() {
    let mut args = std::env::args();
    let _ = args.next();
    let file_path_str = args.next().expect("No file path provided");
    let file_path = std::path::Path::new(&file_path_str);
    if !file_path.exists() {
        panic!("File does not exist: {}", file_path_str);
    }
    let is_debug = args.next().map_or(false, |arg| arg == "--debug");
    let bytes = std::fs::read(file_path).expect("Failed to read file");

    let mut mem = mem::Memory::new();
    mem.load(0, &bytes);
    mem.using_macroquad = !is_debug;
    let mut cpu = cpu::Cpu::new();

    if is_debug {
        cpu.debugger(&mut mem);
    } else {
        cpu.run(&mut mem).await;
    }
}

fn window_config() -> macroquad::window::Conf {
    let char_width = 10.0;
    let char_height = 24.0;
    let window_width = (char_width * mem::TEXT_GRID_WIDTH as f32).ceil() as i32;
    let window_height = (char_height * mem::TEXT_GRID_HEIGHT as f32).ceil() as i32;

    macroquad::window::Conf {
        window_title: "Screen".to_string(),
        window_width,
        window_height,
        window_resizable: false,
        ..Default::default()
    }
}
