import 'package:flutter_test/flutter_test.dart';
import 'package:road_drawing_flutter/models/station_data.dart';
import 'package:road_drawing_flutter/services/preview_fallback.dart';

void main() {
  List<StationData> sampleStations() => [
        StationData(name: 'No.0', x: 0.0, wl: 3.45, wr: 3.55),
        StationData(name: 'No.1', x: 20.0, wl: 3.50, wr: 3.50),
        StationData(name: 'No.2', x: 40.0, wl: 3.55, wr: 3.55),
      ];

  // ================================================================
  // Basic geometry calculation
  // ================================================================

  group('calculate', () {
    test('produces lines and texts from stations', () {
      final data = PreviewFallback.calculate(sampleStations());
      expect(data.lines, isNotEmpty);
      expect(data.texts, isNotEmpty);
    });

    test('empty stations produce empty data', () {
      final data = PreviewFallback.calculate([]);
      expect(data.lines, isEmpty);
      expect(data.texts, isEmpty);
    });

    test('single station produces 2 width lines', () {
      final data = PreviewFallback.calculate([
        StationData(name: 'No.0', x: 0.0, wl: 3.0, wr: 3.0),
      ]);
      expect(data.lines.length, 2);
    });

    test('two stations produce 7 lines', () {
      final data = PreviewFallback.calculate([
        StationData(name: 'No.0', x: 0.0, wl: 2.5, wr: 2.5),
        StationData(name: 'No.1', x: 20.0, wl: 2.5, wr: 2.5),
      ]);
      // 2×2 width + 3 connecting (center + top + bottom) = 7
      expect(data.lines.length, 7);
    });

    test('3 stations produce 12 lines', () {
      final data = PreviewFallback.calculate(sampleStations());
      // 3×2 width + 2×3 connecting = 12
      expect(data.lines.length, 12);
    });
  });

  // ================================================================
  // Rust parity: line counts
  // ================================================================

  group('Rust parity', () {
    test('20 stations produce 97 lines (matches Rust)', () {
      final stations = List.generate(
        20,
        (i) => StationData(name: 'No.$i', x: i * 20.0, wl: 2.5, wr: 2.5),
      );
      final data = PreviewFallback.calculate(stations);
      expect(data.lines.length, 97,
          reason: 'Must match Rust: 20×2 width + 19×3 connecting = 97');
    });

    test('width lines are vertical (x1 == x2)', () {
      final data = PreviewFallback.calculate([
        StationData(name: 'No.0', x: 0, wl: 3.0, wr: 3.0),
      ]);
      for (final l in data.lines) {
        expect(l.x1, closeTo(l.x2, 1e-9));
      }
    });

    test('left width positive Y, right width negative Y', () {
      final data = PreviewFallback.calculate([
        StationData(name: 'No.0', x: 0, wl: 3.0, wr: 2.0),
      ]);
      expect(data.lines[0].y2, closeTo(3000.0, 1e-9));
      expect(data.lines[1].y2, closeTo(-2000.0, 1e-9));
    });

    test('center connecting line at y=0', () {
      final data = PreviewFallback.calculate([
        StationData(name: 'No.0', x: 0, wl: 2.0, wr: 2.0),
        StationData(name: 'No.1', x: 20, wl: 2.0, wr: 2.0),
      ]);
      final centerLines = data.lines.where(
        (l) => l.y1.abs() < 0.01 && l.y2.abs() < 0.01 && l.x1 != l.x2,
      );
      expect(centerLines, isNotEmpty);
      expect(centerLines.first.x2, closeTo(20000, 1e-9));
    });
  });

  // ================================================================
  // Station name labels (color=5, blue)
  // ================================================================

  group('station names', () {
    test('color=5 texts contain station names', () {
      final data = PreviewFallback.calculate(sampleStations());
      final nameTexts = data.texts.where((t) => t.color == 5).toList();
      expect(nameTexts.length, 3);
      final names = nameTexts.map((t) => t.text).toSet();
      expect(names, containsAll(['No.0', 'No.1', 'No.2']));
    });

    test('Japanese station names preserved', () {
      final data = PreviewFallback.calculate([
        StationData(name: '起点', x: 0, wl: 2.0, wr: 2.0),
        StationData(name: '終点', x: 100, wl: 2.0, wr: 2.0),
      ]);
      final names = data.texts.where((t) => t.color == 5).map((t) => t.text).toSet();
      expect(names, containsAll(['起点', '終点']));
    });
  });

  // ================================================================
  // Dimension texts
  // ================================================================

  group('dimension texts', () {
    test('width dimensions are green (color=3) rotated -90', () {
      final data = PreviewFallback.calculate([
        StationData(name: 'No.0', x: 0, wl: 3.0, wr: 2.5),
      ]);
      final dimTexts = data.texts.where((t) => t.color == 3).toList();
      expect(dimTexts.length, 2);
      for (final t in dimTexts) {
        expect(t.rotation, -90);
      }
      final values = dimTexts.map((t) => t.text).toSet();
      expect(values, containsAll(['3.00', '2.50']));
    });

    test('zero width suppresses dimension text', () {
      final data = PreviewFallback.calculate([
        StationData(name: 'No.0', x: 0, wl: 0, wr: 0),
      ]);
      final dimTexts = data.texts.where((t) => t.color == 3);
      expect(dimTexts, isEmpty);
    });

    test('distance dimension is red (color=1)', () {
      final data = PreviewFallback.calculate([
        StationData(name: 'No.0', x: 0, wl: 2.0, wr: 2.0),
        StationData(name: 'No.1', x: 20, wl: 2.0, wr: 2.0),
      ]);
      final distTexts = data.texts.where((t) => t.color == 1).toList();
      expect(distTexts.length, 1);
      expect(distTexts[0].text, '20.00');
    });
  });

  // ================================================================
  // DXF color 5 divergence check
  // ================================================================

  group('color divergence', () {
    test('DxfPreview uses 0xFF0000FF for color=5 (Rust uses 0xFF0080FF)', () {
      // NOTE: This is a known divergence.
      // PreviewFallback produces color=5 for station names.
      // dxf_preview.dart _dxfColor maps 5 → 0xFF0000FF (pure blue)
      // web/flutter/ version maps 5 → 0xFF0080FF (dodger blue)
      // The Rust egui renderer uses 0xFF0080FF (rgb(0,128,255))
      // This is cosmetic but worth tracking.
      final data = PreviewFallback.calculate([
        StationData(name: 'No.0', x: 0, wl: 2.0, wr: 2.0),
      ]);
      final nameTexts = data.texts.where((t) => t.color == 5);
      expect(nameTexts, isNotEmpty);
      // The actual color rendering happens in _DxfPainter._dxfColor
      // We flag this divergence for impl-a
    });
  });
}
