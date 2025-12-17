use std::time::Instant;

use quickcap::capscreen::windows::capscreen;
use rayon::prelude::*;

fn main() {
    let start_time = Instant::now();
    let mut frame = capscreen().unwrap();
    println!("capscreen time: {:?}", start_time.elapsed());
    println!("frame.data {:?}",&frame.data[0..8]);
    let start_time = Instant::now();
    frame.data.par_chunks_exact_mut(4).for_each(|pixel| {
        pixel.swap(0, 2);
        pixel[3] = 255;
    });
    println!("rayon simd time: {:?}", start_time.elapsed());
    println!("frame.data {:?}",&frame.data[0..8]);
}
