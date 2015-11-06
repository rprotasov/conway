extern crate rand;
extern crate image as im;
extern crate find_folder;
extern crate piston_window;

use piston_window::*;
use im::{GenericImage, ImageBuffer, Rgba};
use rand::{Rng};
use std::ops::Range;

const GAME_WIDTH:     u32 = 500;
const GAME_HEIGHT:    u32 = 500;
const CELL_DIMENSION: u32 = 10;

enum State {
    Running,
    Pause,
    Restart,
}

enum Settup {
    Random,
    Pulsar,
    GliderGun,
    Still,
    Toad,
}

impl Settup {
    fn make(&self, population: &mut Population) {
        match *self {
            Settup::Random => {
                let mut rng = rand::thread_rng();
                for index in population.range.clone() {
                    population.status[index] = match rng.gen() {
                        true => true,
                        false => false,
                    };
                }
            },
            Settup::Pulsar => {

            },
            Settup::GliderGun => {

            },
            Settup::Still => {

            },
            Settup::Toad => {
                //
                // TODO: Update so its places
                //       toads in the center of the graph
                //
                population.status[1272] = true;
                let mut b = population.adjacent[1272][3];
                let mut t = population.adjacent[b][0];
                for _ in 0..2 {
                    population.status[b] = true;
                    population.status[t] = true;
                    b = population.adjacent[b][3];
                    t = population.adjacent[t][3];
                }
                population.status[t] = true;
            },
        }
    }
}


struct Population {
    generation: u32,
    range: Range<usize>,
    adjacent: Vec<Vec<usize>>,
    status: Vec<bool>,
    next: Vec<bool>,
}

impl Population {
    fn new(size: usize) -> Population {
        let width:        i64             = (size as f64).sqrt() as i64;
        let mut status:   Vec<bool>       = Vec::with_capacity(size);
        let mut next:     Vec<bool>       = Vec::with_capacity(size);
        let mut adjacent: Vec<Vec<usize>> = Vec::with_capacity(size);

        for _ in 0..size {
            status.push(false);
            next.push(false);
            adjacent.push(Vec::with_capacity(6));
        }

        let index_for = |x: i64, y: i64| {
            (width * y + x) as usize
        };

        let top = |x: i64, y: i64| {
            index_for(x, match y == 0 {
                true => width - 1,
                false => y - 1,
            })
        };

        let down = |x: i64, y: i64| {
            index_for(x, match y == width - 1 {
                true => 0,
                false => y + 1,
            })
        };

        let left = |x: i64, y: i64| {
            index_for( match x == 0 {
                true => width - 1,
                false => x - 1,
            }, y)
        };

        let right = |x: i64, y: i64| {
            index_for( match x == width - 1 {
                true => 0,
                false => x + 1
            }, y)
        };

        //
        // Adjacent cells are represented as:
        // [
        //     Top, Down,
        //     Left, Right,
        //     Top-Left, Top-Right,
        //     Bottom-Left, Bottom-Right
        // ]
        //

        let mut index: usize = 0;
        for y in 0..width {
            for x in 0..width {
                adjacent[index].push(top(x, y));
                adjacent[index].push(down(x, y));
                adjacent[index].push(left(x, y));
                adjacent[index].push(right(x, y));
                index += 1;
            }
        }

        for c in 0..size {
            let tl = adjacent[adjacent[c][0]][2];
            let tr = adjacent[adjacent[c][0]][3];
            let bl = adjacent[adjacent[c][1]][2];
            let br = adjacent[adjacent[c][1]][3];
            adjacent[c].push(
                tl
            );
            adjacent[c].push(
                tr
            );
            adjacent[c].push(
                bl
            );
            adjacent[c].push(
                br
            );
        }

        Population {
            generation: 1,
            range: (0..size),
            adjacent: adjacent,
            status: status,
            next: next,
        }
    }

    fn update(&mut self) {
        for c in self.range.clone() {
            let mut score = 0;
            for a in &self.adjacent[c] {
                if self.status[*a] { score += 1; }
            }
            self.next[c] = match self.status[c] {
                true => {
                    match score {
                        2 | 3 => true,
                        _ => false,
                    }
                },
                false => {
                    match score {
                        3 => true,
                        _ => false,
                    }
                },
            };
        }
        for c in self.range.clone() {
            self.status[c] = self.next[c];
        }
        self.generation += 1;
    }

    fn restart(&mut self) {
        self.generation = 1;
        for index in self.range.clone() {
            self.next[index] = false;
            self.status[index] = false;
        }
    }
}

struct App<'p> {
    population: &'p mut Population,
    state: State,
    settup: Settup,
}

impl<'p> App<'p> {
    fn new(population: &'p mut Population, settup: Settup) -> App<'p> {
        settup.make(population);
        App {
            population: population,
            state: State::Pause,
            settup: settup,
        }
    }

    fn update(&mut self) {
        match self.state {
            State::Pause => { return },
            State::Restart => {
                self.population.restart();
                self.settup.make(&mut self.population);
                self.state = State::Running;
            },
            State::Running => { },
        }

        self.population.update();
    }

    fn render(&self) -> Vec<(Vec<u32>, [u8; 4])> {
        let mut y: u32;
        let mut x: u32;
        let mut canvas: Vec<(Vec<u32>, [u8; 4])> = Vec::with_capacity(
            self.population.range.end as usize
        );
        for i in self.population.range.clone() {
            let color: [u8; 4];
            if self.population.status[i] {
                color = [82, 82, 82, 255];
            } else { color = [255; 4]; }

            y = (i as u32 * CELL_DIMENSION / GAME_WIDTH) * CELL_DIMENSION;
            x = i as u32 * CELL_DIMENSION - y / CELL_DIMENSION * GAME_WIDTH;

            for w in 0..CELL_DIMENSION {
                for h in 0..CELL_DIMENSION {
                    canvas.push(
                        (vec!(x + w, y + h), color)
                    );
                }
            }
        }
        canvas
    }

    fn start(&mut self) {
        self.state = State::Running;
    }

    fn pause(&mut self) {
        self.state = State::Pause;
    }
}

fn main() {
    let opengl = OpenGL::V3_2;
    let window: PistonWindow =
        WindowSettings::new("Rust - Conway",
            (GAME_WIDTH, GAME_HEIGHT))
        .exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();
    let mut canvas = ImageBuffer::new(GAME_WIDTH, GAME_HEIGHT);
    let mut texture = Texture::from_image(
        &mut *window.factory.borrow_mut(),
        &canvas,
        &TextureSettings::new()
    ).unwrap();

    let mut pop =
        Population::new(
            ((GAME_WIDTH * GAME_HEIGHT) / (CELL_DIMENSION * CELL_DIMENSION)) as usize
        );

    let mut app = App::new(
        &mut pop,
        Settup::Random,
    );

    app.start();

    for e in window {
        e.draw_2d(|c, g| {
            clear([1.0; 4], g);
            image(&texture, c.transform, g);
            app.update();
            for p in app.render() {
                canvas.put_pixel(
                    p.0[0],
                    p.0[1],
                    Rgba(p.1)
                );
            }
            texture.update(
                &mut *e.factory.borrow_mut(),
                &canvas,
            ).unwrap();
        });
    }
}
