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
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[test]
    #[wasm_bindgen_test]
    fn new() {
        let result = Counter::new();
        assert_eq!(result, Counter { count: 0 });
    }
}
