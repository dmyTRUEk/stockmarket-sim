//! stockmarket simulator

use std::hint;

// use clap::Parser;
use minifb::{Key, Window, WindowOptions};
use rand::{distr::weighted::WeightedIndex, rng, rngs::StdRng, Rng, SeedableRng};

mod font_rendering;

use font_rendering::*;


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



fn main() {
	// let CliArgs { seed, verbose } = CliArgs::parse();

	// let mut rng = StdRng::seed_from_u64(string_to_u64(&seed));

	let (mut w, mut h) = (1600, 900);
	let mut buffer: Vec<u32> = vec![BLACK.0; w * h];

	let mut window = Window::new(
		&format!("Stockmarket Simulator v{}", env!("CARGO_PKG_VERSION")),
		w, h,
		WindowOptions {
			resize: true,
			..WindowOptions::default()
		}
	).expect("unable to create window");

	window.set_target_fps(60);
	window.update_with_buffer(&buffer, w, h).expect(UNABLE_TO_UPDATE_WINDOW_BUFFER);

	let mut stock = Stock::new();
	let mut player_data = PlayerData {
		money: 10_000.,
		shares: vec![0],
	};

	struct Msg {
		text: String,
		color: Color,
		timeout: i32,
	}

	let mut is_paused: bool = false;
	let mut scale: u32 = 1;
	let mut msgs: Vec<Msg> = vec![];

	while window.is_open() && !window.is_key_down(Key::Escape) {
		let mut is_redraw_needed: bool = false;

		// handle resizing
		(w, h) = window.get_size();
		let new_size = w * h;
		if new_size != buffer.len() {
			buffer.resize(new_size, 0);
			//if verbose { println!("Resized to {w}x{h}") }
			is_redraw_needed = true;
		}


		if window.is_key_pressed_once(Key::Space) {
			is_paused = !is_paused;
		}

		if window.is_key_pressed_repeat(Key::I) {
			if scale > 1 {
				scale -= 1;
				is_redraw_needed = true;
			}
		}
		if window.is_key_pressed_repeat(Key::O) {
			scale += 1;
			is_redraw_needed = true;
		}

		if window.is_key_pressed_repeat(Key::Q) {
			let ssv = stock.get_last_value();
			if player_data.money > ssv {
				player_data.money -= ssv;
				player_data.shares[0] += 1;
			} else {
				msgs.push(Msg{ text:"NOT ENOUGH MONEY TO BUY A SHARE".to_string(), color:RED, timeout:45 });
			}
			is_redraw_needed = true;
		}
		if window.is_key_pressed_repeat(Key::A) {
			let ssv = stock.get_last_value();
			if player_data.shares[0] > 0 {
				player_data.shares[0] -= 1;
				player_data.money += ssv;
			} else {
				msgs.push(Msg{ text:"DONT HAVE A SHARE TO SELL".to_string(), color:RED, timeout:45 });
			}
			is_redraw_needed = true;
		}


		if !is_paused {
			stock.next();
			is_redraw_needed = true;
		}


		if is_redraw_needed {
			buffer = vec![BLACK.0; w * h];
			let hf = h as float;

			// dbg!(stock.history.len(), stock.history.last().unwrap());
			let local_history = stock.get_recent_history_scaled(w as u32 - 8*6*2, scale);
			// dbg!(local_history.len(), w);
			assert!(local_history.len() < w);

			fn value_to_screen_h(value: float, v_min: float, v_max: float, hf: float) -> u32 {
				(hf * (1. - unlerp(value, v_min, v_max))) as u32
			}

			{
				// render delta bars
				let v_min: float = stock.history.min();
				let v_max: float = stock.history.max();
				let mut v_prev: float = *local_history.first().unwrap();
				// let mut h_prev: usize = (hf * (1. - unlerp(v_prev, v_min, v_max))) as usize;
				let mut h_prev: u32 = value_to_screen_h(v_prev, v_min, v_max, hf);
				for (x, v) in local_history.iter().skip(1).enumerate() {
					let x = x as u32;
					let h_curr: u32 = value_to_screen_h(*v, v_min, v_max, hf);
					if *v > v_prev {
						for y in h_curr..h_prev {
							buffer[((w as u32) * y + x) as usize] = GREEN.0;
						}
					} else {
						for y in h_prev..h_curr {
							buffer[((w as u32) * y + x) as usize] = RED.0;
						}
					}
					h_prev = h_curr;
					v_prev = *v;
				}
			}

			let buffer_wh = (w as u32, h as u32);

			{
				buffer.render_text(
					&format!("MONEY  $: {:.2}", player_data.money),
					(10, 10),
					WHITE,
					3,
					buffer_wh,
				);
				buffer.render_text(
					&format!("SHARES $: {:.2}", player_data.get_total_shares_value(vec![stock.get_last_value()])),
					(10, 10+30),
					WHITE,
					3,
					buffer_wh,
				);
				buffer.render_text(
					&format!("TOTAL  $: {:.2}", player_data.get_total_value(vec![stock.get_last_value()])),
					(10, 10+30*2),
					WHITE,
					3,
					buffer_wh,
				);
				buffer.render_text(
					&format!("SHARES N: {}", player_data.shares[0]),
					(10, 10+30*3),
					WHITE,
					3,
					buffer_wh,
				);
				let mut msg_indices_to_remove: Vec<u32> = Vec::with_capacity(1);
				for (i, msg) in msgs.iter_mut().enumerate() {
					let i = i as u32;
					buffer.render_text(
						&msg.text,
						(10, 10+30*4 + 5 + 20*(i as i32)),
						msg.color,
						2,
						buffer_wh,
					);
					msg.timeout -= 1;
					if msg.timeout <= 0 {
						msg_indices_to_remove.push(i);
					}
				}
				// TODO(bugfix): msgs doesnt dissapear when paused
				for i in msg_indices_to_remove.into_iter().rev() {
					msgs.remove(i as usize);
				}
			}

			{
				let current_stock_price = stock.history.last().unwrap();
				let all_time_high = stock.history.max();
				let all_time_low = stock.history.min();
				let csp_y = value_to_screen_h(*current_stock_price, all_time_low, all_time_high, hf) as i32;
				let csp_y = csp_y.min((h as i32) - 6*2);
				buffer.render_text(
					&format!("{current_stock_price}"),
					((w as i32)-8*6*2, csp_y),
					WHITE,
					2,
					buffer_wh,
				);

				// local (on screen) high:
				let local_high = local_history.max();
				let lh_y = if local_high != all_time_high {
					let lh_y = value_to_screen_h(local_high, all_time_low, all_time_high, hf) as i32;
					let lh_y = lh_y.min(csp_y - 6*2); // prevent overlapping
					buffer.render_text(
						&format!("{local_high}"),
						((w as i32)-8*6*2, lh_y),
						CYAN,
						2,
						buffer_wh,
					);
					lh_y
				} else { 0 };
				// all time high:
				let mut ath_y = 0_i32;
				// if local_high != all_time_high {
				// 	ath_y = ath_y.min(lh_y - 6*2); // prevent overlapping
				// }
				ath_y = ath_y.min(if local_high != all_time_high { lh_y } else { csp_y } - 6*2); // prevent overlapping
				buffer.render_text(
					&format!("{all_time_high}"),
					((w as i32)-8*6*2, ath_y),
					GREEN,
					2,
					buffer_wh,
				);

				// local (on screen) low:
				let local_low = local_history.min();
				let ll_y = if local_low != all_time_low {
					let ll_y = value_to_screen_h(local_low, all_time_low, all_time_high, hf) as i32 - 6*2;
					let ll_y = ll_y.max(csp_y + 6*2); // prevent overlapping
					buffer.render_text(
						&format!("{local_low}"),
						((w as i32)-8*6*2, ll_y),
						MAGENTA,
						2,
						buffer_wh,
					);
					ll_y
				} else { 0 };
				// all time low:
				let mut atl_y = (h as i32) - 6*2;
				// if local_low != all_time_low {
				// 	atl_y = atl_y.max(ll_y + 6*2); // prevent overlapping
				// }
				atl_y = atl_y.max(if local_low != all_time_low { ll_y } else { csp_y } + 6*2); // prevent overlapping
				buffer.render_text(
					&format!("{all_time_low}"),
					((w as i32)-8*6*2, atl_y),
					RED,
					2,
					buffer_wh,
				);
			}
		} // end of render

		window.update_with_buffer(&buffer, w, h).expect(UNABLE_TO_UPDATE_WINDOW_BUFFER);
	} // end of main loop
}

const UNABLE_TO_UPDATE_WINDOW_BUFFER: &str = "unable to update window buffer";



#[derive(Clone, Copy)]
struct Color(u32);

const BLACK: Color = Color(0x000000);
const WHITE: Color = Color(0xffffff);

const RED  : Color = Color(0xff0000);
const GREEN: Color = Color(0x00ff00);
const BLUE : Color = Color(0x0000ff);

const CYAN   : Color = Color(0x00ffff);
const MAGENTA: Color = Color(0xff00ff);
const YELLOW : Color = Color(0xffff00);



struct PlayerData {
	money: float,
	/// number of bought shares in company id = index
	shares: Vec<u32>,
}
impl PlayerData {
	fn get_total_shares_value(&self, single_share_values: Vec<float>) -> float {
		assert_eq!(self.shares.len(), single_share_values.len());
		self.shares.iter().zip(single_share_values)
			.map(|(n, ssv)| (*n as float) * ssv)
			.sum()
	}

	fn get_total_value(&self, single_share_values: Vec<float>) -> float {
		self.money + self.get_total_shares_value(single_share_values)
	}
}



struct Stock {
	history: Vec<float>,
	// TODO(optim): min/max value as fields, updated in `.next()`
}
impl Stock {
	fn new() -> Self {
		let mut rng = rng();
		let init_value: float = {
			//              0     1     2     3     4     5
			let weights = [1e-3, 1e-2, 3e-1, 1e-1, 1e-2, 1e-3];
			let distr = WeightedIndex::new(weights).unwrap();
			let num_of_digits = rng.sample(distr);
			rng.random_range(1. .. 9.999) * 10_f64.powi(num_of_digits as i32)
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

	fn next(&mut self) {
		let prev_value = self.get_last_value();
		let mut rng = rng();
		let sign = if rng.random_bool(0.5) { 1. } else { -1. };
		let step = rng.random_range(-3. .. 6.);
		let step = sign * 2_f64.powf(step);
		let new_value = prev_value + step;
		self.history.push(new_value);
	}

	fn get_recent_history_scaled(&self, max_num_of_prices: u32, scale: u32) -> Vec<float> {
		let history = &self.history;
		let history: Vec<float> = history.chunks(scale as usize)
			.map(|chunk| *chunk.last().unwrap())
			.collect();
		let index_of_first: usize = history.len().saturating_sub((max_num_of_prices as usize)-1);
		history[index_of_first..].to_vec()
	}
}





fn unlerp(v: float, v_min: float, v_max: float) -> float {
	// lerp: v = v_min * (1-t) + v_max * t
	(v - v_min) / (v_max - v_min) // = t
}



#[allow(non_camel_case_types)]
type float = f64;



trait PartialCmpMinMax<T: PartialOrd> {
	fn min(&self) -> T;
	fn max(&self) -> T;
}
impl PartialCmpMinMax<float> for Vec<float> {
	fn min(&self) -> float {
		*self.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
	}
	fn max(&self) -> float {
		*self.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
	}
}



trait WindowExtIsKeyPressed {
	fn is_key_pressed_once(&self, key: Key) -> bool;
	fn is_key_pressed_repeat(&self, key: Key) -> bool;
}
impl WindowExtIsKeyPressed for Window {
	fn is_key_pressed_once(&self, key: Key) -> bool {
		self.is_key_pressed(key, minifb::KeyRepeat::No)
	}
	fn is_key_pressed_repeat(&self, key: Key) -> bool {
		self.is_key_pressed(key, minifb::KeyRepeat::Yes)
	}
}

