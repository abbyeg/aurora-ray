use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

// High-speed RNG algorithm (xoshiro256**)
struct Xoshiro256 {
    state: [u64; 4],
}

impl Xoshiro256 {
    fn new(seed: u64) -> Self {
        let mut state = [0; 4];
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        
        // Seed all state elements differently
        for i in 0..4 {
            hasher.write_u64(i as u64);
            state[i] = hasher.finish();
        }
        let mut rng = Xoshiro256 { state };
        // Warm-up the state
        for _ in 0..16 {
            rng.next();
        }
        rng
    }

    #[inline]
    fn next(&mut self) -> u64 {
        let result = self.state[0].wrapping_add(self.state[3]).rotate_left(23).wrapping_add(self.state[0]);
        let t = self.state[1] << 17;
        
        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];
        
        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);
        
        result
    }

    // Generate f64 in [0, 1) with full 53-bit precision
    #[inline]
    fn next_f64(&mut self) -> f64 {
        let u = self.next() >> 11;
        (u as f64) * (1.0 / (1u64 << 53) as f64)
    }
}

thread_local! {
    static THREAD_RNG: RefCell<Xoshiro256> = {
        // Unique seed per thread combining time and thread ID
        let thread_id = thread::current().id().as_u64().get();
        let time_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        
        RefCell::new(Xoshiro256::new(time_seed ^ thread_id))
    };
}

pub fn random_f64() -> f64 {
    THREAD_RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        rng.next_f64()
    })
}

pub fn random_in_range(min: f64, max: f64) -> f64 {
    THREAD_RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        let (a, b) = if min <= max { (min, max) } else { (max, min) };
        let val = rng.next_f64();
        a + val * (b - a)
    })
}
