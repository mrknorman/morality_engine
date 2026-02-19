#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FooterNavConfig {
    pub start_index: usize,
    pub count: usize,
}

impl FooterNavConfig {
    pub const fn new(start_index: usize, count: usize) -> Self {
        Self { start_index, count }
    }

    pub fn contains(self, index: usize) -> bool {
        self.count > 0 && index >= self.start_index && index < self.start_index + self.count
    }

    pub fn cycle(self, index: usize, forward: bool) -> Option<usize> {
        if !self.contains(index) {
            return None;
        }
        let local = index - self.start_index;
        let next_local = if forward {
            (local + 1) % self.count
        } else {
            (local + self.count - 1) % self.count
        };
        Some(self.start_index + next_local)
    }
}

#[cfg(test)]
mod tests {
    use super::FooterNavConfig;

    #[test]
    fn cycles_footer_indices_with_wrap() {
        let footer = FooterNavConfig::new(10, 3);
        assert_eq!(footer.cycle(10, true), Some(11));
        assert_eq!(footer.cycle(12, true), Some(10));
        assert_eq!(footer.cycle(10, false), Some(12));
        assert_eq!(footer.cycle(9, true), None);
    }
}
