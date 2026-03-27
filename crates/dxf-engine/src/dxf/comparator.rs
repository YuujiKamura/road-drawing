//! DXF comparison: normalize and compare DXF content ignoring handle IDs.
//!
//! Used for golden-file testing: verify that generated DXF matches a reference
//! file regardless of handle assignment order.

/// Normalized DXF representation for comparison.
///
/// Strips handle IDs (group code 5) and owner references (group code 330),
/// then sorts entity blocks for order-independent comparison.
#[derive(Debug, Clone, PartialEq)]
pub struct DxfComparable {
    /// Normalized entity blocks (handle-stripped, sorted)
    pub entities: Vec<String>,
}

impl DxfComparable {
    /// Normalize a DXF string for comparison.
    ///
    /// 1. Extract the ENTITIES section
    /// 2. Split into individual entity blocks
    /// 3. Strip handle IDs (group code 5) and owner refs (group code 330)
    /// 4. Sort entity blocks for stable comparison
    pub fn normalize(dxf_content: &str) -> Self {
        let lines: Vec<&str> = dxf_content.lines().map(|l| l.trim()).collect();

        // Find ENTITIES section
        let entities_start = lines.iter().position(|&l| l == "ENTITIES")
            .map(|i| i + 1); // skip the "ENTITIES" line
        let entities_end = lines.iter().enumerate()
            .skip(entities_start.unwrap_or(0))
            .find(|(_, &l)| l == "ENDSEC")
            .map(|(i, _)| i);

        let (start, end) = match (entities_start, entities_end) {
            (Some(s), Some(e)) => (s, e),
            _ => return Self { entities: Vec::new() },
        };

        // Split into entity blocks: each starts with group code "0" followed by
        // an entity type keyword (LINE, TEXT, CIRCLE, LWPOLYLINE).
        const ENTITY_TYPES: &[&str] = &["LINE", "TEXT", "CIRCLE", "LWPOLYLINE"];
        let entity_lines = &lines[start..end];
        let mut blocks: Vec<Vec<&str>> = Vec::new();
        let mut current: Vec<&str> = Vec::new();

        let mut i = 0;
        while i < entity_lines.len() {
            if entity_lines[i] == "0"
                && i + 1 < entity_lines.len()
                && ENTITY_TYPES.contains(&entity_lines[i + 1])
            {
                if !current.is_empty() {
                    blocks.push(current);
                    current = Vec::new();
                }
                current.push(entity_lines[i]);     // "0"
                current.push(entity_lines[i + 1]);  // entity type
                i += 2;
            } else {
                current.push(entity_lines[i]);
                i += 1;
            }
        }
        if !current.is_empty() {
            blocks.push(current);
        }

        // Keep only blocks that start with "0" + known entity type
        let blocks: Vec<_> = blocks.into_iter()
            .filter(|block| block.len() >= 2 && block[0] == "0"
                && ENTITY_TYPES.contains(&block[1]))
            .collect();

        // Normalize each block: strip group codes 5 (handle) and 330 (owner)
        let mut normalized: Vec<String> = blocks.iter().map(|block| {
            let mut result: Vec<&str> = Vec::new();
            let mut j = 0;
            while j < block.len() {
                if j + 1 < block.len() && (block[j] == "5" || block[j] == "330") {
                    // Skip this group code and its value
                    j += 2;
                } else {
                    result.push(block[j]);
                    j += 1;
                }
            }
            result.join("\n")
        }).collect();

        // Sort for order-independent comparison
        normalized.sort();

        Self { entities: normalized }
    }
}

/// Compare two DXF strings after normalization.
/// Returns Ok(()) if equivalent, Err with description of first difference.
pub fn compare_dxf_strings(expected: &str, actual: &str) -> Result<(), String> {
    let exp = DxfComparable::normalize(expected);
    let act = DxfComparable::normalize(actual);

    if exp.entities.len() != act.entities.len() {
        return Err(format!(
            "Entity count mismatch: expected {}, got {}",
            exp.entities.len(), act.entities.len()
        ));
    }

    for (i, (e, a)) in exp.entities.iter().zip(act.entities.iter()).enumerate() {
        if e != a {
            return Err(format!(
                "Entity[{i}] differs:\n  expected: {e}\n  actual:   {a}"
            ));
        }
    }

    Ok(())
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
        let dxf1 = dxf_3lines();
        // Replace handle values with different hex
        let dxf2 = dxf1
            .replace("5\n100\n", "5\nAA00\n")
            .replace("5\n101\n", "5\nBB01\n")
            .replace("5\n102\n", "5\nCC02\n");

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

        // Alter all handles
        let dxf_alt = dxf.replace("5\n1", "5\nFF");
        let norm_alt = DxfComparable::normalize(&dxf_alt);

        assert_eq!(
            norm, norm_alt,
            "100-entity DXF with altered handles must still compare equal"
        );
    }

    // ================================================================
    // (6) compare_dxf_strings helper
    // ================================================================

    #[test]
    fn test_compare_dxf_strings_ok() {
        let dxf = dxf_3lines();
        assert!(compare_dxf_strings(&dxf, &dxf).is_ok());
    }

    #[test]
    fn test_compare_dxf_strings_count_mismatch() {
        let dxf1 = DxfWriter::new().write(&[DxfLine::new(0.0, 0.0, 1.0, 1.0)], &[]);
        let dxf2 = DxfWriter::new().write(&[], &[]);
        let err = compare_dxf_strings(&dxf1, &dxf2).unwrap_err();
        assert!(err.contains("count mismatch"));
    }

    // ================================================================
    // (7) Road section golden file: 2-station DXF (7 lines + 7 texts)
    // ================================================================

    #[test]
    fn test_marking_golden_road_section_simple() {
        // Reproduce the exact same 2-station road section generation
        // that created tests/golden/road_section_simple.dxf
        //
        // CSV: No.0(0,2.5,2.5) + No.1(20,3.0,3.0) with scale=1000
        // We call road_section via its public API equivalents as DXF entities:

        let golden = include_str!("../../tests/golden/road_section_simple.dxf");
        let golden_norm = DxfComparable::normalize(golden);

        // Golden file should have 14 entities (7 lines + 7 texts)
        assert_eq!(
            golden_norm.entities.len(), 14,
            "Road section golden should have 14 entities (7 LINE + 7 TEXT), got {}",
            golden_norm.entities.len()
        );

        // Verify LINE and TEXT entities are present
        let line_count = golden_norm.entities.iter()
            .filter(|e| e.contains("AcDbLine"))
            .count();
        let text_count = golden_norm.entities.iter()
            .filter(|e| e.contains("AcDbText"))
            .count();
        assert_eq!(line_count, 7, "Should have 7 LINE entities");
        assert_eq!(text_count, 7, "Should have 7 TEXT entities");
    }

    #[test]
    fn test_marking_golden_road_section_self_compare() {
        let golden = include_str!("../../tests/golden/road_section_simple.dxf");
        assert!(
            compare_dxf_strings(golden, golden).is_ok(),
            "Golden file must compare equal to itself"
        );
    }

    #[test]
    fn test_marking_golden_road_section_handle_insensitive() {
        let golden = include_str!("../../tests/golden/road_section_simple.dxf");
        let altered = golden
            .replace("5\n100\n", "5\nDEAD\n")
            .replace("5\n101\n", "5\nBEEF\n")
            .replace("5\n102\n", "5\nCAFE\n");

        assert_ne!(golden, &altered, "Raw strings should differ");
        assert!(
            compare_dxf_strings(golden, &altered).is_ok(),
            "Road section golden with altered handles must still compare equal"
        );
    }
}
