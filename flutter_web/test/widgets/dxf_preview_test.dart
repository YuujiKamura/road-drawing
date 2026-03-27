import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:road_drawing_flutter/models/preview_data.dart';
import 'package:road_drawing_flutter/widgets/dxf_preview.dart';

void main() {
  Widget buildPreview(PreviewData data) {
    return MaterialApp(
      home: Scaffold(
        body: SizedBox(
          width: 800,
          height: 600,
          child: DxfPreview(data: data),
        ),
      ),
    );
  }

  PreviewData sampleData() => PreviewData(
        lines: [
          PreviewLine(x1: 0, y1: 0, x2: 0, y2: 3000, color: 7),
          PreviewLine(x1: 0, y1: 0, x2: 0, y2: -3000, color: 7),
          PreviewLine(x1: 0, y1: 0, x2: 20000, y2: 0, color: 7),
        ],
        texts: [
          PreviewText(text: 'No.0', x: 0, y: -3500, rotation: 0, height: 350, color: 5),
        ],
      );

  // ================================================================
  // Rendering
  // ================================================================

  group('rendering', () {
    testWidgets('renders ClipRect+CustomPaint with data', (tester) async {
      await tester.pumpWidget(buildPreview(sampleData()));
      expect(find.byType(ClipRect), findsOneWidget);
      expect(find.byType(CustomPaint), findsWidgets);
    });

    testWidgets('renders with empty data without crash', (tester) async {
      await tester.pumpWidget(buildPreview(PreviewData.empty()));
      // Should still render (dark background only)
      expect(find.byType(ClipRect), findsOneWidget);
    });

    testWidgets('updates when data changes', (tester) async {
      await tester.pumpWidget(buildPreview(PreviewData.empty()));
      await tester.pumpWidget(buildPreview(sampleData()));
      expect(find.byType(ClipRect), findsOneWidget);
    });
  });

  // ================================================================
  // Japanese text
  // ================================================================

  group('Japanese text', () {
    testWidgets('renders Japanese station names without crash', (tester) async {
      final data = PreviewData(
        lines: [PreviewLine(x1: 0, y1: 0, x2: 0, y2: 2000, color: 7)],
        texts: [
          PreviewText(text: '起点', x: 0, y: -2500, rotation: 0, height: 350, color: 5),
          PreviewText(text: '終点', x: 100000, y: -2500, rotation: 0, height: 350, color: 5),
        ],
      );
      await tester.pumpWidget(buildPreview(data));
      expect(find.byType(ClipRect), findsOneWidget);
    });

    testWidgets('handles mixed ASCII and Japanese', (tester) async {
      final data = PreviewData(
        lines: [PreviewLine(x1: 0, y1: 0, x2: 0, y2: 2000, color: 7)],
        texts: [
          PreviewText(text: 'No.0+5.5', x: 0, y: -2500, rotation: 0, height: 350, color: 5),
          PreviewText(text: '交差点A', x: 50000, y: -2500, rotation: 0, height: 350, color: 5),
        ],
      );
      await tester.pumpWidget(buildPreview(data));
      expect(find.byType(ClipRect), findsOneWidget);
    });
  });

  // ================================================================
  // Pan & zoom interaction
  // ================================================================

  group('interaction', () {
    testWidgets('has GestureDetector for pan/zoom', (tester) async {
      await tester.pumpWidget(buildPreview(sampleData()));
      expect(find.byType(GestureDetector), findsWidgets);
    });
  });

  // ================================================================
  // Large dataset
  // ================================================================

  group('performance', () {
    testWidgets('handles 100-line dataset without crash', (tester) async {
      final lines = List.generate(
        100,
        (i) => PreviewLine(
          x1: i * 20000.0, y1: 0,
          x2: i * 20000.0, y2: 3000,
          color: 7,
        ),
      );
      final data = PreviewData(lines: lines, texts: []);
      await tester.pumpWidget(buildPreview(data));
      expect(find.byType(ClipRect), findsOneWidget);
    });
  });
}
