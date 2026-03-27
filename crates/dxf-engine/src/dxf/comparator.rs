//! DXF comparison: normalize and compare DXF content ignoring handle IDs.
//!
//! Used for golden-file testing: verify that generated DXF matches a reference
//! file regardless of handle assignment order.

/// Normalized DXF representation for comparison.
///
/// Strips handle IDs and sorts entities so that two DXF outputs with
/// identical geometry but different handle assignments compare equal.
#[derive(Debug, Clone, PartialEq)]
pub struct DxfComparable {
    /// Normalized entity lines (handle-stripped, sorted)
    pub entities: Vec<String>,
}

impl DxfComparable {
    /// Normalize a DXF string for comparison.
    ///
    /// 1. Strip handle IDs (group code 5)
    /// 2. Extract entity blocks
    /// 3. Sort entities for order-independent comparison
    pub fn normalize(_dxf_content: &str) -> Self {
        todo!("Issue #7: implement DXF normalization for golden-file comparison")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DxfLine, DxfWriter};

    /// Generate a DXF string from 3 lines
    fn dxf_3lines() -> String {
        let lines = vec![
            DxfLine::new(0.0, 0.0, 100.0, 0.0),
            DxfLine::new(100.0, 0.0, 100.0, 100.0),
            DxfLine::new(100.0, 100.0, 0.0, 0.0),
        ];
        let writer = DxfWriter::new();
        writer.write(&lines, &[])
    }

    // ================================================================
    // (1) Golden file: hand-written 3-LINE DXF matches generated
    // ================================================================

    #[test]
    fn test_marking_golden_3lines_matches_generated() {
        let generated = dxf_3lines();
        let golden = include_str!("../../tests/golden/simple_3lines.dxf");

        let gen_norm = DxfComparable::normalize(&generated);
        let gold_norm = DxfComparable::normalize(golden);

        assert_eq!(
            gen_norm, gold_norm,
            "Generated DXF must match golden file after normalization"
        );
    }

    // ================================================================
    // (2) Same DXF with different handle IDs must compare equal
    // ================================================================

    #[test]
    fn test_marking_golden_different_handles_equal() {
        // Generate twice — handle IDs will be identical in practice,
        // so we manually alter handles in the second copy
        let dxf1 = dxf_3lines();
        let dxf2 = dxf1.replace("5\n1", "5\nAA").replace("5\n2", "5\nBB").replace("5\n3", "5\nCC");

        // Verify handles actually differ
        assert_ne!(dxf1, dxf2, "Raw DXF strings should differ (different handles)");

        let norm1 = DxfComparable::normalize(&dxf1);
        let norm2 = DxfComparable::normalize(&dxf2);

        assert_eq!(
            norm1, norm2,
            "DXF with different handle IDs must compare equal after normalization"
        );
    }

    // ================================================================
    // (3) DXF with 1 extra LINE must compare not-equal
    // ================================================================

    #[test]
    fn test_marking_golden_extra_line_not_equal() {
        let lines_3 = vec![
            DxfLine::new(0.0, 0.0, 100.0, 0.0),
            DxfLine::new(100.0, 0.0, 100.0, 100.0),
            DxfLine::new(100.0, 100.0, 0.0, 0.0),
        ];
        let lines_4 = vec![
            DxfLine::new(0.0, 0.0, 100.0, 0.0),
            DxfLine::new(100.0, 0.0, 100.0, 100.0),
            DxfLine::new(100.0, 100.0, 0.0, 0.0),
            DxfLine::new(50.0, 50.0, 200.0, 200.0), // extra
        ];

        let writer = DxfWriter::new();
        let dxf_3 = writer.write(&lines_3, &[]);
        let dxf_4 = writer.write(&lines_4, &[]);

        let norm_3 = DxfComparable::normalize(&dxf_3);
        let norm_4 = DxfComparable::normalize(&dxf_4);

        assert_ne!(
            norm_3, norm_4,
            "DXF with extra entity must NOT compare equal"
        );
    }

    // ================================================================
    // (4) Empty DXF edge case
    // ================================================================

    #[test]
    fn test_marking_golden_empty_dxf() {
        let writer = DxfWriter::new();
        let empty_dxf = writer.write(&[], &[]);

        let norm = DxfComparable::normalize(&empty_dxf);

        // Empty DXF should normalize to empty entities
        assert!(
            norm.entities.is_empty(),
            "Empty DXF should have 0 normalized entities, got {}",
            norm.entities.len()
        );
    }

    // ================================================================
    // (5) 100 entities stress test
    // ================================================================

    #[test]
    fn test_marking_golden_100_entities_stress() {
        let lines: Vec<DxfLine> = (0..100)
            .map(|i| {
                let f = i as f64;
                DxfLine::new(f, f, f + 10.0, f + 10.0)
            })
            .collect();

        let writer = DxfWriter::new();
        let dxf = writer.write(&lines, &[]);

        let norm = DxfComparable::normalize(&dxf);

        assert_eq!(
            norm.entities.len(),
            100,
            "100-entity DXF should normalize to 100 entities"
        );

        // Same content, different handles → still equal
        let dxf_alt = dxf.replace("5\n1\n", "5\nFF01\n");
        let norm_alt = DxfComparable::normalize(&dxf_alt);

        assert_eq!(
            norm, norm_alt,
            "100-entity DXF with altered handles must still compare equal"
        );
    }
}
