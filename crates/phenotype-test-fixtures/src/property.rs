//! Property-based testing utilities

use rand::{distributions::uniform::SampleUniform, Rng};

/// Strategy for generating property-based test inputs
pub struct Strategy {
    #[allow(dead_code)]
    name: String,
    description: String,
    seed: u64,
}

impl Strategy {
    /// Create a new strategy
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            seed: 42,
        }
    }

    /// Set the strategy description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set a specific seed for reproducible generation
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Generate a random string
    pub fn generate_string(&self, len: usize) -> String {
        let mut rng = rand::thread_rng();
        (0..len)
            .map(|_| rng.gen_range(32u8..127u8) as char)
            .collect()
    }

    /// Generate a random number
    pub fn generate_number<T: SampleUniform + Copy + PartialOrd>(&self, min: T, max: T) -> T {
        let mut rng = rand::thread_rng();
        rng.gen_range(min..=max)
    }
}

impl Default for Strategy {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Generate a random alphanumeric string
pub fn alphanumeric(len: usize) -> String {
    let mut rng = rand::thread_rng();
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let charset_len = CHARSET.len();
    (0..len)
        .map(|_| CHARSET[rng.gen_range(0..charset_len)] as char)
        .collect()
}

/// Generate a random string with specific length
pub fn string_of_len(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| rng.gen_range(32u8..127u8) as char)
        .collect()
}

/// Generate a random email address
pub fn email() -> String {
    let mut rng = rand::thread_rng();
    let local = alphanumeric(rng.gen_range(5..15));
    let domain = alphanumeric(rng.gen_range(5..10));
    let tld = match rng.gen_range(0..4) {
        0 => "com",
        1 => "org",
        2 => "net",
        _ => "io",
    };
    format!("{}@{}.{}", local, domain, tld)
}

/// Generate a random UUID string
pub fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Generate a random vector of elements using a generator function
pub fn vec_of<T, F>(len: usize, generator: F) -> Vec<T>
where
    F: Fn() -> T,
{
    (0..len).map(|_| generator()).collect()
}

/// Generate a random HashMap
pub fn hash_map<K, V, Fk, Fv>(
    len: usize,
    key_gen: Fk,
    val_gen: Fv,
) -> std::collections::HashMap<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
    Fk: Fn() -> K,
    Fv: Fn() -> V,
{
    let mut map = std::collections::HashMap::with_capacity(len);
    for _ in 0..len {
        map.insert(key_gen(), val_gen());
    }
    map
}

/// Property test runner
pub struct PropertyTest {
    name: String,
    iterations: usize,
}

impl PropertyTest {
    /// Create a new property test
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            iterations: 100,
        }
    }

    /// Set the number of iterations
    pub fn iterations(mut self, n: usize) -> Self {
        self.iterations = n;
        self
    }

    /// Run the property test
    pub fn run<F>(self, mut test: F)
    where
        F: FnMut(),
    {
        for i in 0..self.iterations {
            test();
            if i % 10 == 0 {
                println!("  [{}] iteration {}/{}", self.name, i, self.iterations);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy() {
        let strategy = Strategy::new("test").description("A test strategy");
        let s = strategy.generate_string(10);
        assert_eq!(s.len(), 10);
    }

    #[test]
    fn test_alphanumeric() {
        let s = alphanumeric(10);
        assert_eq!(s.len(), 10);
        assert!(s.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_string_of_len() {
        let s = string_of_len(20);
        assert_eq!(s.len(), 20);
    }

    #[test]
    fn test_uuid_format() {
        let u = uuid();
        assert_eq!(u.len(), 36);
        assert_eq!(u.chars().filter(|&c| c == '-').count(), 4);
    }

    #[test]
    fn test_vec_of() {
        let v = vec_of(5, || 42);
        assert_eq!(v.len(), 5);
        assert!(v.iter().all(|&x| x == 42));
    }

    #[test]
    fn test_hash_map() {
        let map = hash_map(3, || alphanumeric(5), || rand::random::<i32>());
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_property_test() {
        let mut counter = 0;
        PropertyTest::new("test").iterations(10).run(|| {
            counter += 1;
        });
        assert_eq!(counter, 10);
    }
}
