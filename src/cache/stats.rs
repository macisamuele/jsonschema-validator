use crate::cache::Stats;

impl Default for Stats {
    fn default() -> Self {
        Self { hits: 0, misses: 0 }
    }
}

#[allow(unsafe_code)]
unsafe impl Send for Stats {}

#[allow(unsafe_code)]
unsafe impl Sync for Stats {}

impl Stats {
    pub fn register_hit(&mut self) {
        self.hits += 1;
    }

    pub fn register_miss(&mut self) {
        self.misses += 1;
    }

    pub fn get_hits(&self) -> usize {
        self.hits
    }

    pub fn get_misses(&self) -> usize {
        self.misses
    }

    pub fn clear(&mut self) {
        self.hits = 0;
        self.misses = 0;
    }
}
