//! stockmarket simulator

// use clap::Parser;
use minifb::{Key, Window, WindowOptions};
use rand::{distr::weighted::WeightedIndex, rng, rngs::StdRng, Rng, SeedableRng};


// #[derive(Parser, Debug)]
// #[clap(
// 	about,
// 	author,
// 	version,
// 	help_template = "\
// 		{before-help}{name} v{version}\n\
// 		\n\
// 		{about}\n\
// 		\n\
// 		Author: {author}\n\
// 		\n\
// 		{usage-heading} {usage}\n\
// 		\n\
// 		{all-args}{after-help}\
// 	",
// )]
// struct CliArgs {
// 	/// seed
// 	#[arg(short='s', long, default_value="stockmarket-sim")]
// 	seed: String,
//
// 	// TODO
// 	#[arg(short='v', long, default_value_t=false)]
// 	verbose: bool,
// }



fn main() -> Result<(), u8> {
	// let CliArgs { seed, verbose } = CliArgs::parse();

	// let mut rng = StdRng::seed_from_u64(string_to_u64(&seed));

	let (mut w, mut h) = (1600, 900);
	let mut buffer: Vec<u32> = vec![BLACK; w * h];

	let mut window = Window::new(
		"stockmarket-sim",
		w, h,
		WindowOptions {
			resize: true,
			// scale_mode: ScaleMode::Stretch,
			..WindowOptions::default()
		}
	).expect("unable to create window");

	window.set_target_fps(60);
	window.update_with_buffer(&buffer, w, h).expect(UNABLE_TO_UPDATE_WINDOW_BUFFER);

	let mut stock = Stock::new();

	while window.is_open() && !window.is_key_down(Key::Escape) {
		let mut is_redraw_needed: bool = false;

		(w, h) = window.get_size();
		let new_size = w * h;
		if new_size != buffer.len() {
			buffer.resize(new_size, 0);
			//if verbose { println!("Resized to {w}x{h}") }
			is_redraw_needed = true;
		}

		// if window.is_key_pressed_(Key::Q) {
		// 	todo!()
		// }

		stock.next();
		is_redraw_needed = true;

		if is_redraw_needed {
			buffer = vec![BLACK; w * h];
			let hf = h as float;

			dbg!(stock.history.len(), stock.history.last().unwrap());
			let history: &[float] = &stock.history[stock.history.len().saturating_sub(w-1)..];
			// dbg!(history.len(), w);
			assert!(history.len() < w);

			let v_min: float = stock.get_min_value();
			let v_max: float = stock.get_max_value();
			// let max_diff: float = v_max - v_min;
			let mut v_prev: float = *history.first().unwrap();
			let mut h_prev: usize = (hf * (1. - unlerp(v_prev, v_min, v_max))) as usize;
			for (x, v) in history.iter().skip(1).enumerate() {
				let h_curr = (hf * (1. - unlerp(*v, v_min, v_max))) as usize;
				// let diff = v - v_prev;
				// dbg!(diff);
				if *v > v_prev {
					for y in h_curr..h_prev {
						buffer[w * y + x] = GREEN;
					}
				} else {
					for y in h_prev..h_curr {
						buffer[w * y + x] = RED;
					}
				}
				h_prev = h_curr;
				v_prev = *v;
			}
		}

		window.update_with_buffer(&buffer, w, h).expect(UNABLE_TO_UPDATE_WINDOW_BUFFER);
	}

	Ok(())
}

const UNABLE_TO_UPDATE_WINDOW_BUFFER: &str = "unable to update window buffer";

const BLACK: u32 = 0x000000;
const WHITE: u32 = 0xffffff;

const RED  : u32 = 0xff0000;
const GREEN: u32 = 0x00ff00;
const BLUE : u32 = 0x0000ff;



struct Stock {
	history: Vec<float>,
}
impl Stock {
	fn new() -> Self {
		let mut rng = rng();
		let init_value: float = {
			//              0     1     2     3     4     5
			let weights = [1e-3, 1e-2, 3e-1, 1e-1, 1e-2, 1e-3];
			let distr = WeightedIndex::new(weights).unwrap();
			let num_of_digits = rng.sample(distr);
			rng.random_range(1. .. 9.999) * 10_f32.powi(num_of_digits as i32)
		};
		Self::from_init_value(init_value)
	}

	fn from_init_value(value: float) -> Self {
		Self {
			history: vec![value],
		}
	}

	fn get_last_value(&self) -> float {
		*self.history.last().unwrap()
	}
	fn get_max_value(&self) -> float {
		*self.history.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
	}
	fn get_min_value(&self) -> float {
		*self.history.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
	}

	fn next(&mut self) {
		let prev_value = self.get_last_value();
		let mut rng = rng();
		let sign = if rng.random_bool(0.5) { 1. } else { -1. };
		let step = rng.random_range(-2. .. 7.);
		let step = sign * 2_f32.powf(step);
		let new_value = prev_value + step;
		self.history.push(new_value);
	}
}



fn unlerp(v: float, v_min: float, v_max: float) -> float {
	// v = v_min * (1-t) + v_max * t
	(v - v_min) / (v_max - v_min) // = t
}




#[allow(non_camel_case_types)]
type float = f32;



trait WindowExtIsKeyPressed_ {
	fn is_key_pressed_(&self, key: Key) -> bool;
}
impl WindowExtIsKeyPressed_ for Window {
	fn is_key_pressed_(&self, key: Key) -> bool {
		self.is_key_pressed(key, minifb::KeyRepeat::No)
	}
}

