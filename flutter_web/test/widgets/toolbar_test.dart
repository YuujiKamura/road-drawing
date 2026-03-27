import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:road_drawing_flutter/widgets/toolbar.dart';

void main() {
  Widget buildToolbar({
    VoidCallback? onAddRow,
    VoidCallback? onDeleteRow,
    VoidCallback? onPreview,
    VoidCallback? onDownload,
  }) {
    return MaterialApp(
      home: Scaffold(
        body: Toolbar(
          onAddRow: onAddRow ?? () {},
          onDeleteRow: onDeleteRow ?? () {},
          onPreview: onPreview ?? () {},
          onDownload: onDownload ?? () {},
        ),
      ),
    );
  }

  group('rendering', () {
    testWidgets('displays all buttons', (tester) async {
      await tester.pumpWidget(buildToolbar());
      expect(find.text('+ Row'), findsOneWidget);
      expect(find.text('- Row'), findsOneWidget);
      expect(find.text('Preview'), findsOneWidget);
      expect(find.text('DXF'), findsOneWidget);
    });

    testWidgets('displays Grid Editor title', (tester) async {
      await tester.pumpWidget(buildToolbar());
      expect(find.text('Grid Editor'), findsOneWidget);
    });
  });

  group('callbacks', () {
    testWidgets('+ Row button calls onAddRow', (tester) async {
      int callCount = 0;
      await tester.pumpWidget(buildToolbar(onAddRow: () => callCount++));
      await tester.tap(find.text('+ Row'));
      expect(callCount, 1);
    });

    testWidgets('- Row button calls onDeleteRow', (tester) async {
      int callCount = 0;
      await tester.pumpWidget(buildToolbar(onDeleteRow: () => callCount++));
      await tester.tap(find.text('- Row'));
      expect(callCount, 1);
    });

    testWidgets('Preview button calls onPreview', (tester) async {
      int callCount = 0;
      await tester.pumpWidget(buildToolbar(onPreview: () => callCount++));
      await tester.tap(find.text('Preview'));
      expect(callCount, 1);
    });

    testWidgets('DXF button calls onDownload', (tester) async {
      int callCount = 0;
      await tester.pumpWidget(buildToolbar(onDownload: () => callCount++));
      await tester.tap(find.text('DXF'));
      expect(callCount, 1);
    });
  });
}
