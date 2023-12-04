#[derive(PartialEq, Debug)]
pub struct Counter {
    count: u8,
}

impl Counter {
    pub fn new() -> Counter {
        Counter { count: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let result = Counter::new();
        assert_eq!(result, Counter { count: 0 });
    }
}
