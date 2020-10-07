#[macro_use] extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate rand;

use std::time::Instant;
use gfx::traits::FactoryExt;
use gfx::Device;
use gfx_window_glutin as gfx_glutin;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}



#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    dirty: bool,
}

impl Universe {

    fn new(width: u32, height: u32) -> Universe {
        // let mut rng = rand::thread_rng();
        return Universe {
            width,
            height,
            cells: (0..(width*height)).map(|i| {
                // if i % 2 == 0 || i % 7 == 0 {
                if rand::random::<u8>()%2 == 1 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            }).collect(),
            dirty: true,
        };
    }

    fn get_cell_state(&self, row: u32, column: u32) -> Cell {
        let idx = self.get_index(row, column);
        return self.cells[idx];
    }

    fn live_neighbors(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for drow in ([-1, 0, 1] as [i32; 3]).iter().cloned() {
            for dcol in ([-1, 0, 1] as [i32; 3]).iter().cloned() {
                if (drow == 0 && dcol == 0) ||
                    (drow == -1 && row == 0) ||
                    (drow == 1 && row == self.height - 1) ||
                    (dcol == -1 && column == 0) ||
                    (dcol == 1  && column == self.width - 1) {
                    continue;
                }
                let idx = self.get_index(((row as i32) + drow) as u32, ((column as i32) + dcol) as u32);
                count += self.cells[idx] as u8;
            }
        }
        return count;
    }    

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }
        

    fn tick(&mut self) {
        self.dirty = false;
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);            
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbors(row, col);

                let next_cell_state = match (cell, live_neighbors) {
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    (Cell::Dead, 3) => Cell::Alive,
                    (otherwise_set_same_state, _) => otherwise_set_same_state,
                };

                if next[idx] != next_cell_state {
                    self.dirty = true;
                }
                next[idx] = next_cell_state;
            }
        }

        self.cells = next;
    }

    fn debug_print(&self) {
        for row in 0..self.height {
            for col in 0..self.width {
                let cell_state = self.get_cell_state(row, col);
    
                if cell_state == Cell::Alive { print!("*"); } else { print!(" "); }
            }
            print!("\n");
        }
        println!("-----------------------------------------------------\n");        
    }
}


#[derive(Debug, Clone, Copy)]
struct Square {
    pub pos: (f32, f32),
    pub size: f32,
    pub color: [f32; 3]
}

impl Square {
    pub fn new(pos: (f32, f32), size: f32, color: [f32; 3]) -> Square {
        Self {
            pos,
            size,
            color
        }
    }

    pub fn extend_indices_vector(&self, is: &mut Vec<u32>, i: u32) {
        is.extend(&[
            4*i, 4*i + 1, 4*i + 2, 4*i + 2, 4*i + 3, 4*i
        ]);
    }

    pub fn extend_vertices_vector(&self, vs: &mut Vec<Vertex>, aspect_ratio: f32) {
        let (hx, hy);
        if aspect_ratio > 1.0 {
            hx = self.size / aspect_ratio;
            hy = self.size;
        }
        else {
            hx = self.size;
            hy = self.size * aspect_ratio;
        }

        vs.extend(&[
            Vertex { pos: [self.pos.0, self.pos.1], color: self.color },
            Vertex { pos: [self.pos.0 + hx, self.pos.1], color: self.color },
            Vertex { pos: [self.pos.0 + hx, self.pos.1 - hy], color: self.color },
            Vertex { pos: [self.pos.0, self.pos.1 - hy], color: self.color },
        ]);
    }    
}

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WHITE: [f32; 3] = [1.0, 1.0, 1.0];
const RED: [f32; 4] = [1.0, 0.00, 0.00, 1.0];
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];


const RED3: [f32; 3] = [1.0, 0.0, 0.0];
const WHITE3: [f32; 3] = [1.0, 1.0, 1.0];

const SQUARE_SIZE: f32 = 0.01;
const COLUMNS: u32 = (2.0/SQUARE_SIZE) as u32;
const ROWS: u32 = (2.0/SQUARE_SIZE) as u32;

pub fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let fullscreen = Some(glutin::window::Fullscreen::Borderless(event_loop.primary_monitor()));
    let builder = glutin::window::WindowBuilder::new()
        .with_title("Game of Life".to_string())
        .with_fullscreen(fullscreen.clone());
        
    let context = glutin::ContextBuilder::new()
        .with_gl(glutin::GL_CORE);

    let (window, mut device, mut factory, mut main_color, mut main_depth) = gfx_glutin::init::<gfx::format::Srgba8, gfx::format::DepthStencil, ()>(builder, context, &event_loop).unwrap();
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let pso = factory.create_pipeline_simple(
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/rect_150.glslv")),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/rect_150.glslf")),
        pipe::new()
    ).unwrap();

    let vertices: Vec<Vertex> = Vec::new();
    let indices: Vec<u16> = Vec::new();
    let (vertex_buffer, mut slice) = factory.create_vertex_buffer_with_slice(&vertices, &*indices);

    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        out: main_color.clone()
    };


    let mut total_frames = 0;
    let start_time = Instant::now();
    let mut last_time = Instant::now();
    
    let mut aspect_ratio = 0.0;
    let mut needs_update = false;

    let mut universe: Universe = Universe::new((2.0/SQUARE_SIZE) as u32, (2.0/SQUARE_SIZE) as u32);
    
    // let mut normal_text = gfx_text::new(factory.clone()).unwrap();
    let mut normal_text = gfx_text::RendererBuilder::new(factory.clone())
        .with_size(50)
        .with_font("fonts/Roboto-Bold.ttf")
        .build()
        .unwrap();
 
    

    event_loop.run(move |event, _, control_flow| {
        *control_flow = glutin::event_loop::ControlFlow::Poll;
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested =>  *control_flow = glutin::event_loop::ControlFlow::Exit,
                glutin::event::WindowEvent::KeyboardInput {
                    input:
                        glutin::event::KeyboardInput {
                            virtual_keycode: Some(virtual_code),
                            state,
                            ..
                        },
                    ..
                } => match (virtual_code, state) {
                    (glutin::event::VirtualKeyCode::Escape, _) =>  *control_flow = glutin::event_loop::ControlFlow::Exit,
                    (glutin::event::VirtualKeyCode::F, glutin::event::ElementState::Pressed) => {
                        if !window.window().fullscreen().is_some() {
                            window.window().set_fullscreen(fullscreen.clone());
                        } else {
                            window.window().set_fullscreen(None);
                        }
                        universe = Universe::new((2.0/SQUARE_SIZE) as u32, (2.0/SQUARE_SIZE) as u32);
                    }
                    (glutin::event::VirtualKeyCode::R, glutin::event::ElementState::Pressed) => {
                        universe = Universe::new((2.0/SQUARE_SIZE) as u32, (2.0/SQUARE_SIZE) as u32);
                    }
                    _ => (),
                },
                glutin::event::WindowEvent::Resized(physical_size) => {
                    gfx_glutin::update_views(&window, &mut main_color, &mut main_depth);
                    aspect_ratio = physical_size.width as f32 / physical_size.height as f32;
                    needs_update = true;
                },
                _ => (),
            },
            _ => (),
        }

        let mut squares: Vec<Square> = Vec::new();  

        let time_diff = (Instant::now() - last_time).as_secs_f32();
        if time_diff > 0.1 {
            // println!("Tick {:?}\n", (Instant::now() - last_time));
            last_time = Instant::now();
            universe.tick();
            needs_update = true;
            // universe.debug_print();
        }

        if needs_update {
            for row in 0..universe.height {
                for col in 0..universe.width {
                    let cell_state = universe.get_cell_state(row, col);
                    if cell_state == Cell::Alive {
                        // The top left of the screen is -1.0, 1.0 and the bottom right is 1.0, -1.0
                        // So I add -1.0 to the row or col * size of the square to get the x or y coordinate
                        squares.push(Square::new((-1.0 + (col as f32) * SQUARE_SIZE,-1.0 + (row as f32) * SQUARE_SIZE), SQUARE_SIZE, RED3));
                    } else {
                    }
                }
            }

            let mut vs: Vec<Vertex> = Vec::new();
            let mut is: Vec<u32> = Vec::new();
            for (index, square) in squares.iter().enumerate() {
                square.extend_vertices_vector(&mut vs, aspect_ratio);
                square.extend_indices_vector(&mut is, index as u32);
            }
            let (vbuf, sl) = factory.create_vertex_buffer_with_slice(&vs, &*is);

            data.vbuf = vbuf;
            data.out = main_color.clone();
            slice = sl;            
            needs_update = false
        }


        total_frames += 1;
        let fps = (total_frames as f32) / (Instant::now() - start_time).as_secs_f32();
        let fps_string = format!("FPS: {:.2}", fps);
        normal_text.add(&fps_string, [100, 100], GREEN);
        
        encoder.clear(&main_color, BLACK);

        // Draw Squares
        encoder.draw(&slice, &pso, &data);

        // Draw Text
        normal_text.draw(&mut encoder, &main_color).unwrap();

        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    });

}
