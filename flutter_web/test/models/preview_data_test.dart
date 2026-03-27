import 'package:flutter_test/flutter_test.dart';
import 'package:road_drawing_flutter/models/preview_data.dart';

void main() {
  // ================================================================
  // PreviewData.fromJson
  // ================================================================

  group('PreviewData.fromJson', () {
    test('parses lines and texts from JSON', () {
      const json = '{"lines":[{"x1":0,"y1":0,"x2":1000,"y2":0,"color":7}],'
          '"texts":[{"text":"No.0","x":0,"y":-500,"rotation":0,"height":350,"color":5}]}';
      final data = PreviewData.fromJson(json);
      expect(data.lines.length, 1);
      expect(data.texts.length, 1);
      expect(data.lines[0].x2, closeTo(1000, 1e-9));
      expect(data.texts[0].text, 'No.0');
      expect(data.texts[0].color, 5);
    });

    test('parses empty arrays', () {
      const json = '{"lines":[],"texts":[]}';
      final data = PreviewData.fromJson(json);
      expect(data.lines, isEmpty);
      expect(data.texts, isEmpty);
    });

    test('handles missing optional fields with defaults', () {
      const json = '{"lines":[{"x1":0,"y1":0,"x2":100,"y2":100}],'
          '"texts":[{"text":"A","x":0,"y":0}]}';
      final data = PreviewData.fromJson(json);
      expect(data.lines[0].color, 7); // default
      expect(data.texts[0].rotation, 0); // default
      expect(data.texts[0].height, 350); // default
      expect(data.texts[0].color, 7); // default
    });

    test('handles Japanese text in JSON', () {
      const json = '{"lines":[],"texts":[{"text":"起点","x":0,"y":0,"rotation":0,"height":350,"color":5}]}';
      final data = PreviewData.fromJson(json);
      expect(data.texts[0].text, '起点');
    });
  });

  // ================================================================
  // PreviewData.empty
  // ================================================================

  group('PreviewData.empty', () {
    test('creates empty data', () {
      final data = PreviewData.empty();
      expect(data.lines, isEmpty);
      expect(data.texts, isEmpty);
    });
  });

  // ================================================================
  // PreviewLine.fromJson
  // ================================================================

  group('PreviewLine.fromJson', () {
    test('parses coordinates and color', () {
      final line = PreviewLine.fromJson({
        'x1': 0.0, 'y1': 100.0, 'x2': 500.0, 'y2': -200.0, 'color': 3,
      });
      expect(line.x1, closeTo(0, 1e-9));
      expect(line.y1, closeTo(100, 1e-9));
      expect(line.x2, closeTo(500, 1e-9));
      expect(line.y2, closeTo(-200, 1e-9));
      expect(line.color, 3);
    });

    test('handles integer coordinates', () {
      final line = PreviewLine.fromJson({
        'x1': 0, 'y1': 0, 'x2': 1000, 'y2': 0, 'color': 7,
      });
      expect(line.x1, closeTo(0, 1e-9));
      expect(line.x2, closeTo(1000, 1e-9));
    });
  });
}
