//! DXF handle generator
//!
//! Generates unique entity handles for DXF files.
//! Each entity in a DXF file must have a unique handle (group code 5).

/// Handle generator for DXF entities
///
/// Generates sequential handles starting from a specified value.
/// Handles are output as uppercase hexadecimal strings.
#[derive(Debug, Clone)]
pub struct HandleGenerator {
    next: u32,
}

impl Default for HandleGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl HandleGenerator {
    /// Creates a new HandleGenerator starting at 0x100 (256)
    ///
    /// Starting at 0x100 leaves room for standard DXF objects
    /// (tables, blocks, etc.) which typically use lower handles.
    pub fn new() -> Self {
        Self { next: 0x100 }
    }

    /// Creates a HandleGenerator with a custom starting value
    pub fn with_start(start: u32) -> Self {
        Self { next: start }
    }

    /// Generates the next handle and increments the counter
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> String {
        let handle = self.next;
        self.next = self.next.saturating_add(1);
        format!("{:X}", handle)
    }

    /// Returns the current handle without incrementing
    pub fn current(&self) -> String {
        format!("{:X}", self.next)
    }

    /// Returns the current counter value
    pub fn current_value(&self) -> u32 {
        self.next
    }
}

/// Standard owner handles for DXF structures
pub mod owners {
    /// Model space block record (standard owner for most entities)
    pub const MODEL_SPACE: &str = "1F";

    /// Entities section owner (alternative)
    pub const ENTITIES: &str = "36";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_generator_default() {
        let mut gen = HandleGenerator::new();
        assert_eq!(gen.next(), "100");
        assert_eq!(gen.next(), "101");
        assert_eq!(gen.next(), "102");
    }

    #[test]
    fn test_handle_generator_with_start() {
        let mut gen = HandleGenerator::with_start(0x50);
        assert_eq!(gen.next(), "50");
        assert_eq!(gen.next(), "51");
    }

    #[test]
    fn test_handle_generator_current() {
        let mut gen = HandleGenerator::new();
        assert_eq!(gen.current(), "100");
        gen.next();
        assert_eq!(gen.current(), "101");
    }

    #[test]
    fn test_handle_uniqueness() {
        let mut gen = HandleGenerator::new();
        let mut handles = Vec::new();
        for _ in 0..1000 {
            handles.push(gen.next());
        }
        // Check all handles are unique
        let unique: std::collections::HashSet<_> = handles.iter().collect();
        assert_eq!(unique.len(), 1000);
    }

    #[test]
    fn test_handle_format_uppercase() {
        let mut gen = HandleGenerator::with_start(0xABC);
        assert_eq!(gen.next(), "ABC");
    }

    // ================================================================
    // Boundary values
    // ================================================================

    #[test]
    fn test_handle_generator_start_zero() {
        let mut gen = HandleGenerator::with_start(0);
        assert_eq!(gen.next(), "0");
        assert_eq!(gen.next(), "1");
    }

    #[test]
    fn test_handle_generator_start_one() {
        let mut gen = HandleGenerator::with_start(1);
        assert_eq!(gen.next(), "1");
    }

    #[test]
    fn test_handle_generator_start_max() {
        let mut gen = HandleGenerator::with_start(0xFFFF_FFFE);
        assert_eq!(gen.next(), "FFFFFFFE");
        assert_eq!(gen.next(), "FFFFFFFF");
    }

    #[test]
    fn test_handle_current_value() {
        let gen = HandleGenerator::new();
        assert_eq!(gen.current_value(), 0x100);
    }

    #[test]
    fn test_handle_current_value_after_next() {
        let mut gen = HandleGenerator::new();
        gen.next();
        gen.next();
        assert_eq!(gen.current_value(), 0x102);
    }

    #[test]
    fn test_handle_default_equals_new() {
        let a = HandleGenerator::new();
        let b = HandleGenerator::default();
        assert_eq!(a.current_value(), b.current_value());
    }

    #[test]
    fn test_handle_clone() {
        let mut gen = HandleGenerator::new();
        gen.next();
        let cloned = gen.clone();
        assert_eq!(gen.current_value(), cloned.current_value());
    }

    #[test]
    fn test_handle_clone_independence() {
        let mut gen = HandleGenerator::new();
        let mut cloned = gen.clone();
        gen.next();
        // Cloned should remain at original position
        assert_ne!(gen.current_value(), cloned.current_value());
        assert_eq!(cloned.next(), "100");
    }

    #[test]
    fn test_handle_sequential_1000() {
        let mut gen = HandleGenerator::new();
        let mut prev = 0u32;
        for i in 0..1000 {
            let h = gen.next();
            let val = u32::from_str_radix(&h, 16).unwrap();
            if i > 0 {
                assert_eq!(val, prev + 1);
            }
            prev = val;
        }
    }

    #[test]
    fn test_handle_hex_format_no_prefix() {
        let mut gen = HandleGenerator::new();
        let h = gen.next();
        assert!(!h.starts_with("0x"));
        assert!(!h.starts_with("0X"));
    }

    #[test]
    fn test_owners_constants() {
        assert_eq!(owners::MODEL_SPACE, "1F");
        assert_eq!(owners::ENTITIES, "36");
    }
}
