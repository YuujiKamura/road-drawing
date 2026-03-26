# excel-parser

Excel and CSV parser for road cross-section survey data. Detects named sections within spreadsheets, extracts station names and distance/width columns, determines whether distances are cumulative or incremental, generates missing station names from distance values, and transforms raw rows into `road-section` compatible data. Supports `.xlsx` via calamine and Shift_JIS encoded CSV files.

## Usage

```rust
use std::path::Path;
use excel_parser::transform::{list_sections, extract_and_transform};

let path = Path::new("survey_data.xlsx");

// List available sections in the file
let sections = list_sections(&path);
println!("Sections: {:?}", sections);

// Extract and transform a specific section
let rows = extract_and_transform(&path, &sections[0]).unwrap();
for row in &rows {
    println!("{}: dist={}, L={}, R={}", row.name, row.distance, row.left, row.right);
}
```
