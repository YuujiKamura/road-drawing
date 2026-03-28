/// CSV → DXF conversion pipeline test (WASM mock via PreviewFallback).
///
/// Tests the full pipeline: CSV text → StationData.fromCsv → PreviewFallback.calculate
/// → PreviewData (lines + texts). This mirrors what WASM parse_csv + get_preview_data
/// would produce, validated against known Rust output expectations.
import 'package:flutter_test/flutter_test.dart';
import 'package:road_drawing_flutter/models/station_data.dart';
import 'package:road_drawing_flutter/models/preview_data.dart';
import 'package:road_drawing_flutter/services/preview_fallback.dart';

void main() {
  // ================================================================
  // Full pipeline: CSV text → parse → geometry → preview data
  // ================================================================

  group('CSV → DXF pipeline (WASM mock)', () {
    test('standard 3-station CSV produces correct geometry', () {
      const csv = 'name,x,wl,wr\nNo.0,0,3.45,3.55\nNo.1,20,3.50,3.50\nNo.2,40,3.55,3.55\n';
      final stations = StationData.fromCsv(csv);
      expect(stations.length, 3);

      final preview = PreviewFallback.calculate(stations);
      // 3×2 width + 2×3 connecting = 12 lines (Rust parity)
      expect(preview.lines.length, 12);

      // 3 station names (color=5) + dimension texts
      final nameTexts = preview.texts.where((t) => t.color == 5).toList();
      expect(nameTexts.length, 3);
      expect(nameTexts.map((t) => t.text).toSet(), {'No.0', 'No.1', 'No.2'});
    });

    test('CSV → toCsv → fromCsv roundtrip preserves geometry', () {
      final original = [
        StationData(name: 'No.0', x: 0, wl: 3.45, wr: 3.55),
        StationData(name: 'No.1', x: 20, wl: 3.50, wr: 3.50),
      ];
      final csv = StationData.toCsv(original);
      final restored = StationData.fromCsv(csv);

      final previewOrig = PreviewFallback.calculate(original);
      final previewRestored = PreviewFallback.calculate(restored);

      expect(previewRestored.lines.length, previewOrig.lines.length);
      expect(previewRestored.texts.length, previewOrig.texts.length);

      // Coordinate precision check
      for (int i = 0; i < previewOrig.lines.length; i++) {
        expect(previewRestored.lines[i].x1, closeTo(previewOrig.lines[i].x1, 0.1));
        expect(previewRestored.lines[i].y1, closeTo(previewOrig.lines[i].y1, 0.1));
        expect(previewRestored.lines[i].x2, closeTo(previewOrig.lines[i].x2, 0.1));
        expect(previewRestored.lines[i].y2, closeTo(previewOrig.lines[i].y2, 0.1));
      }
    });

    test('empty CSV produces empty preview', () {
      final stations = StationData.fromCsv('');
      expect(stations, isEmpty);

      final preview = PreviewFallback.calculate(stations);
      expect(preview.lines, isEmpty);
      expect(preview.texts, isEmpty);
    });

    test('header-only CSV produces empty preview', () {
      final stations = StationData.fromCsv('name,x,wl,wr\n');
      expect(stations, isEmpty);

      final preview = PreviewFallback.calculate(stations);
      expect(preview.lines, isEmpty);
    });

    test('single station CSV produces 2 width lines', () {
      const csv = 'No.0,0,3.0,3.0\n';
      final stations = StationData.fromCsv(csv);
      expect(stations.length, 1);

      final preview = PreviewFallback.calculate(stations);
      expect(preview.lines.length, 2);
    });
  });

  // ================================================================
  // Japanese CSV input
  // ================================================================

  group('Japanese CSV pipeline', () {
    test('Japanese header detection + data parsing', () {
      const csv = '測点名,単延長L,幅員W,幅員右\nNo.0,0,3.45,3.55\nNo.1,20,3.50,3.50\n';
      final stations = StationData.fromCsv(csv);
      expect(stations.length, 2);
      expect(stations[0].name, 'No.0');

      final preview = PreviewFallback.calculate(stations);
      expect(preview.lines.length, 7); // 2×2 + 3 connecting
    });

    test('full-width digits in CSV → correct geometry', () {
      const csv = 'No.０,０,３.４５,３.５５\nNo.１,２０,３.５０,３.５０\n';
      final stations = StationData.fromCsv(csv);
      expect(stations.length, 2);
      expect(stations[0].wl, closeTo(3.45, 1e-9));
      expect(stations[1].x, closeTo(20.0, 1e-9));

      final preview = PreviewFallback.calculate(stations);
      expect(preview.lines.length, 7);
    });

    test('Japanese station names appear in preview texts', () {
      const csv = '起点,0,3.0,3.0\n終点,100,3.0,3.0\n';
      final stations = StationData.fromCsv(csv);
      final preview = PreviewFallback.calculate(stations);

      final names = preview.texts.where((t) => t.color == 5).map((t) => t.text).toSet();
      expect(names, containsAll(['起点', '終点']));
    });

    test('full-width minus in width → negative width geometry', () {
      const csv = 'No.0,0,－1.5,0\n';
      final stations = StationData.fromCsv(csv);
      expect(stations[0].wl, closeTo(-1.5, 1e-9));

      final preview = PreviewFallback.calculate(stations);
      // Left width line: y2 = -1.5 * 1000 = -1500 (negative because wl is negative)
      expect(preview.lines[0].y2, closeTo(-1500, 1e-9));
    });
  });

  // ================================================================
  // Coordinate verification (Rust parity)
  // ================================================================

  group('coordinate parity with Rust', () {
    test('scale 1000: x=20m → 20000mm in preview', () {
      const csv = 'No.0,0,2.5,2.5\nNo.1,20,2.5,2.5\n';
      final stations = StationData.fromCsv(csv);
      final preview = PreviewFallback.calculate(stations);

      // Center line: (0,0) → (20000, 0)
      final centerLines = preview.lines.where(
        (l) => l.y1.abs() < 0.01 && l.y2.abs() < 0.01 && l.x1 != l.x2,
      );
      expect(centerLines, isNotEmpty);
      expect(centerLines.first.x2, closeTo(20000, 1e-9));
    });

    test('width 2.5m → 2500mm vertical lines', () {
      const csv = 'No.0,0,2.5,2.5\n';
      final stations = StationData.fromCsv(csv);
      final preview = PreviewFallback.calculate(stations);

      // Left width: (0,0)→(0,2500), Right width: (0,0)→(0,-2500)
      expect(preview.lines[0].y2, closeTo(2500, 1e-9));
      expect(preview.lines[1].y2, closeTo(-2500, 1e-9));
    });

    test('distance dimension text shows 20.00', () {
      const csv = 'No.0,0,2.5,2.5\nNo.1,20,2.5,2.5\n';
      final stations = StationData.fromCsv(csv);
      final preview = PreviewFallback.calculate(stations);

      final distTexts = preview.texts.where((t) => t.color == 1).toList();
      expect(distTexts.length, 1);
      expect(distTexts[0].text, '20.00');
    });

    test('width dimension texts show 2 decimal places', () {
      const csv = 'No.0,0,3.0,2.5\n';
      final stations = StationData.fromCsv(csv);
      final preview = PreviewFallback.calculate(stations);

      final dimTexts = preview.texts.where((t) => t.color == 3).toList();
      expect(dimTexts.length, 2);
      final values = dimTexts.map((t) => t.text).toSet();
      expect(values, containsAll(['3.00', '2.50']));
    });

    test('20 stations → 97 lines (matches Rust test)', () {
      final csv = StringBuffer('name,x,wl,wr\n');
      for (int i = 0; i < 20; i++) {
        csv.writeln('No.$i,${i * 20.0},2.5,2.5');
      }
      final stations = StationData.fromCsv(csv.toString());
      expect(stations.length, 20);

      final preview = PreviewFallback.calculate(stations);
      expect(preview.lines.length, 97,
          reason: '20×2 width + 19×3 connecting = 97 (Rust parity)');
    });
  });

  // ================================================================
  // Edge cases
  // ================================================================

  group('edge cases', () {
    test('zero width stations produce lines but no dimension texts', () {
      const csv = 'No.0,0,0,0\nNo.1,20,0,0\n';
      final stations = StationData.fromCsv(csv);
      final preview = PreviewFallback.calculate(stations);

      // Zero-length width lines still generated
      expect(preview.lines, isNotEmpty);
      // No dimension texts for zero widths
      final dimTexts = preview.texts.where((t) => t.color == 3);
      expect(dimTexts, isEmpty);
    });

    test('mixed valid/invalid rows → valid geometry for good rows', () {
      const csv = 'name,x,wl,wr\nNo.0,0,2.5,2.5\nbad,abc,1,1\nNo.2,40,2.5,2.5\n';
      final stations = StationData.fromCsv(csv);
      expect(stations.length, 2);

      final preview = PreviewFallback.calculate(stations);
      // 2×2 width + 1×3 connecting = 7
      expect(preview.lines.length, 7);
    });

    test('very large CSV (100 stations) processes without error', () {
      final csv = StringBuffer('name,x,wl,wr\n');
      for (int i = 0; i < 100; i++) {
        csv.writeln('No.$i,${i * 20.0},2.5,2.5');
      }
      final stations = StationData.fromCsv(csv.toString());
      expect(stations.length, 100);

      final preview = PreviewFallback.calculate(stations);
      // 100×2 + 99×3 = 497
      expect(preview.lines.length, 497);
    });

    test('extreme coordinate values', () {
      const csv = 'No.0,0,0.001,0.001\nNo.1,99999,99.999,99.999\n';
      final stations = StationData.fromCsv(csv);
      final preview = PreviewFallback.calculate(stations);
      expect(preview.lines, isNotEmpty);
      // No NaN/Infinity
      for (final l in preview.lines) {
        expect(l.x1.isFinite, true);
        expect(l.y1.isFinite, true);
        expect(l.x2.isFinite, true);
        expect(l.y2.isFinite, true);
      }
    });
  });

  // ================================================================
  // JSON interop simulation (mock WASM parse_csv output)
  // ================================================================

  group('JSON interop (WASM mock)', () {
    test('fromJsonList → PreviewFallback matches fromCsv → PreviewFallback', () {
      // Simulate WASM parse_csv returning JSON
      const wasmJson = '[{"name":"No.0","x":0,"wl":3.45,"wr":3.55},'
          '{"name":"No.1","x":20,"wl":3.5,"wr":3.5}]';
      final fromWasm = StationData.fromJsonList(wasmJson);

      // Simulate Dart fallback CSV parse
      const csv = 'No.0,0,3.45,3.55\nNo.1,20,3.5,3.5\n';
      final fromDart = StationData.fromCsv(csv);

      final previewWasm = PreviewFallback.calculate(fromWasm);
      final previewDart = PreviewFallback.calculate(fromDart);

      expect(previewWasm.lines.length, previewDart.lines.length,
          reason: 'WASM JSON path and Dart CSV path must produce same geometry');
      expect(previewWasm.texts.length, previewDart.texts.length);

      for (int i = 0; i < previewWasm.lines.length; i++) {
        expect(previewWasm.lines[i].x1, closeTo(previewDart.lines[i].x1, 0.1));
        expect(previewWasm.lines[i].y1, closeTo(previewDart.lines[i].y1, 0.1));
      }
    });

    test('toCsv output is valid input for fromCsv', () {
      final stations = [
        StationData(name: 'A', x: 0, wl: 1.5, wr: 2.5),
        StationData(name: 'B', x: 30, wl: 3.0, wr: 3.0),
      ];
      final csv = StationData.toCsv(stations);
      final restored = StationData.fromCsv(csv);
      expect(restored.length, 2);
      expect(restored[0].name, 'A');
      expect(restored[1].x, closeTo(30, 1e-9));
    });

    test('toJson roundtrip preserves preview geometry', () {
      final station = StationData(name: 'No.0', x: 10, wl: 2.5, wr: 3.5);
      final json = station.toJson();
      final restored = StationData.fromJson(json);

      final previewOrig = PreviewFallback.calculate([station]);
      final previewRestored = PreviewFallback.calculate([restored]);

      expect(previewOrig.lines.length, previewRestored.lines.length);
      for (int i = 0; i < previewOrig.lines.length; i++) {
        expect(previewRestored.lines[i].x1, closeTo(previewOrig.lines[i].x1, 1e-9));
        expect(previewRestored.lines[i].y2, closeTo(previewOrig.lines[i].y2, 1e-9));
      }
    });
  });
}
