use globset::{GlobBuilder, GlobMatcher};
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

const GENERATION_INTERVAL: Duration = Duration::from_secs(30);
const MAX_GLOBS: usize = 50;

// TODO rename to cache?
pub struct QueryContext {
    globs: FxHashMap<String, (GlobMatcher, u32)>,
    glob_generation: u32,
    last_gc_check: Instant,
    now: Instant,
}

impl Default for QueryContext {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            globs: HashMap::with_capacity_and_hasher(MAX_GLOBS, FxBuildHasher),
            glob_generation: 0,
            last_gc_check: now,
            now,
        }
    }
}

impl QueryContext {
    pub fn check_gc(&mut self) {
        let now = Instant::now();

        if now.duration_since(self.last_gc_check) >= GENERATION_INTERVAL {
            self.glob_generation = self.glob_generation.wrapping_add(1);
            self.last_gc_check = now;
        }

        if self.globs.len() > MAX_GLOBS {
            self.run_gc();
        }
    }

    pub fn run_gc(&mut self) {
        let min_generation = self.glob_generation.saturating_sub(2);
        self.globs.retain(|_, (_, g)| *g >= min_generation);
    }

    pub fn glob(&mut self, pattern: &str) -> &GlobMatcher {
        let current_gen = self.glob_generation;

        if !self.globs.contains_key(pattern) {
            let glob = GlobBuilder::new(pattern)
                .build()
                .unwrap() // FIXME unwrap
                .compile_matcher();
            self.globs.insert(pattern.to_string(), (glob, current_gen));
        } else if let Some((_, g)) = self.globs.get_mut(pattern)
            && *g < current_gen.saturating_sub(1)
        {
            *g = current_gen;
        }

        &self.globs.get(pattern).unwrap().0
    }

    pub fn on_eval_start(&mut self) {
        self.now = Instant::now();
    }

    pub fn now(&self) -> &Instant {
        &self.now
    }
}
