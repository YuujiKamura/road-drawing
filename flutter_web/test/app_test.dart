import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:road_drawing_flutter/app.dart';
import 'package:road_drawing_flutter/widgets/dxf_preview.dart';

void main() {
  Widget buildApp() {
    return const MaterialApp(home: MainLayout());
  }

  void configureView(WidgetTester tester) {
    tester.view.physicalSize = const Size(1600, 900);
    tester.view.devicePixelRatio = 1.0;
  }

  // ================================================================
  // Layout structure
  // ================================================================

  group('MainLayout', () {
    testWidgets('renders grid editor and DXF preview', (tester) async {
      configureView(tester);
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(buildApp());
      expect(find.text('Grid Editor'), findsOneWidget);
      expect(find.byType(DataTable), findsOneWidget);
      expect(find.byType(DxfPreview), findsOneWidget);
    });

    testWidgets('renders toolbar buttons', (tester) async {
      configureView(tester);
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(buildApp());
      expect(find.text('+ Row'), findsOneWidget);
      expect(find.text('- Row'), findsOneWidget);
      expect(find.text('Preview'), findsOneWidget);
      expect(find.text('DXF'), findsOneWidget);
    });

    testWidgets('has DataTable with initial 3 stations', (tester) async {
      configureView(tester);
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(buildApp());
      final dt = tester.widget<DataTable>(find.byType(DataTable));
      expect(dt.rows.length, 3);
    });

    testWidgets('initial preview renders via PreviewFallback', (tester) async {
      configureView(tester);
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(buildApp());
      expect(find.byType(DxfPreview), findsOneWidget);
      // ClipRect inside DxfPreview confirms painting is active
      expect(
        find.descendant(of: find.byType(DxfPreview), matching: find.byType(ClipRect)),
        findsOneWidget,
      );
    });
  });

  // ================================================================
  // Row operations via Toolbar
  // ================================================================

  group('row operations', () {
    testWidgets('+ Row adds a station and updates preview', (tester) async {
      configureView(tester);
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(buildApp());
      var dt = tester.widget<DataTable>(find.byType(DataTable));
      expect(dt.rows.length, 3);

      final addBtn = find.widgetWithText(OutlinedButton, '+ Row');
      await tester.ensureVisible(addBtn);
      await tester.tap(addBtn);
      await tester.pumpAndSettle();

      dt = tester.widget<DataTable>(find.byType(DataTable));
      expect(dt.rows.length, 4,
          reason: 'After tapping + Row, DataTable should have 4 rows (was 3)');
      expect(find.byType(DxfPreview), findsOneWidget);
    });

    testWidgets('- Row removes last station', (tester) async {
      configureView(tester);
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(buildApp());
      final delBtn = find.text('- Row');
      await tester.ensureVisible(delBtn);
      await tester.tap(delBtn);
      await tester.pump();

      final dt = tester.widget<DataTable>(find.byType(DataTable));
      expect(dt.rows.length, 2);
    });

    testWidgets('delete all rows then add does not crash', (tester) async {
      configureView(tester);
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(buildApp());

      final delBtn = find.text('- Row');
      final addBtn = find.text('+ Row');

      // Delete all 3 rows
      for (int i = 0; i < 3; i++) {
        await tester.ensureVisible(delBtn);
        await tester.tap(delBtn);
        await tester.pump();
      }
      var dt = tester.widget<DataTable>(find.byType(DataTable));
      expect(dt.rows, isEmpty);

      // Delete on empty — no crash
      await tester.ensureVisible(delBtn);
      await tester.tap(delBtn);
      await tester.pump();
      dt = tester.widget<DataTable>(find.byType(DataTable));
      expect(dt.rows, isEmpty);

      // Add row back
      await tester.ensureVisible(addBtn);
      await tester.tap(addBtn);
      await tester.pump();
      dt = tester.widget<DataTable>(find.byType(DataTable));
      expect(dt.rows.length, 1);
    });
  });

  // ================================================================
  // E2E: grid edit → preview update
  // ================================================================

  group('E2E grid → preview', () {
    testWidgets('adding multiple rows keeps preview rendering', (tester) async {
      configureView(tester);
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(buildApp());

      final addBtn = find.text('+ Row');
      // Add 2 rows (total 5)
      await tester.ensureVisible(addBtn);
      await tester.tap(addBtn);
      await tester.pump();
      await tester.ensureVisible(addBtn);
      await tester.tap(addBtn);
      await tester.pump();

      final dt = tester.widget<DataTable>(find.byType(DataTable));
      expect(dt.rows.length, 5);
      expect(find.byType(DxfPreview), findsOneWidget);
    });

    testWidgets('Preview button refreshes canvas', (tester) async {
      configureView(tester);
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(buildApp());
      final previewBtn = find.text('Preview');
      await tester.ensureVisible(previewBtn);
      await tester.tap(previewBtn);
      await tester.pump();
      expect(find.byType(DxfPreview), findsOneWidget);
    });
  });
}
