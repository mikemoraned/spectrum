use wasm_bindgen::prelude::wasm_bindgen;

#[derive(PartialEq, Debug)]
#[wasm_bindgen]
pub struct Counter {
    count: u8,
}

#[wasm_bindgen]
impl Counter {
    pub fn new() -> Counter {
        Counter { count: 0 }
    }

    pub fn set_count(&mut self, value: u8) {
        self.count = value;
    }

    pub fn get_count(&self) -> u8 {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[test]
    #[wasm_bindgen_test]
    fn new() {
        let counter = Counter::new();
        assert_eq!(counter, Counter { count: 0 });
    }

    #[test]
    #[wasm_bindgen_test]
    fn set_count() {
        let mut counter = Counter::new();
        counter.set_count(12);
        assert_eq!(counter, Counter { count: 12 });
    }

    #[test]
    #[wasm_bindgen_test]
    fn get_count() {
        let mut counter = Counter::new();
        assert_eq!(counter.get_count(), 0);
        counter.set_count(12);
        assert_eq!(counter.get_count(), 12);
    }
}
