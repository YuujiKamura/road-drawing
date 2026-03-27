import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:road_drawing_flutter/models/station_data.dart';
import 'package:road_drawing_flutter/widgets/grid_editor.dart';

void main() {
  List<StationData> sampleStations() => [
        StationData(name: 'No.0', x: 0.0, wl: 3.45, wr: 3.55),
        StationData(name: 'No.1', x: 20.0, wl: 3.50, wr: 3.50),
      ];

  Widget buildGrid({
    required List<StationData> stations,
    ValueChanged<List<StationData>>? onChanged,
  }) {
    return MaterialApp(
      home: Scaffold(
        body: SizedBox(
          width: 600,
          height: 400,
          child: GridEditor(
            stations: stations,
            onChanged: onChanged ?? (_) {},
          ),
        ),
      ),
    );
  }

  // ================================================================
  // Rendering
  // ================================================================

  group('rendering', () {
    testWidgets('displays column headers', (tester) async {
      await tester.pumpWidget(buildGrid(stations: sampleStations()));
      expect(find.text('測点名'), findsOneWidget);
      expect(find.text('単延長L'), findsOneWidget);
      expect(find.text('幅員W'), findsOneWidget);
      expect(find.text('幅員右'), findsOneWidget);
    });

    testWidgets('displays DataTable', (tester) async {
      await tester.pumpWidget(buildGrid(stations: sampleStations()));
      expect(find.byType(DataTable), findsOneWidget);
    });

    testWidgets('renders correct number of DataRows', (tester) async {
      await tester.pumpWidget(buildGrid(stations: sampleStations()));
      // DataTable should have 2 data rows
      final dt = tester.widget<DataTable>(find.byType(DataTable));
      expect(dt.rows.length, 2);
    });

    testWidgets('empty stations show empty DataTable', (tester) async {
      await tester.pumpWidget(buildGrid(stations: []));
      final dt = tester.widget<DataTable>(find.byType(DataTable));
      expect(dt.rows, isEmpty);
    });

    testWidgets('station name appears in grid', (tester) async {
      await tester.pumpWidget(buildGrid(stations: sampleStations()));
      // TextFormField with initial value 'No.0'
      expect(find.text('No.0'), findsWidgets);
      expect(find.text('No.1'), findsWidgets);
    });

    testWidgets('Japanese station names render correctly', (tester) async {
      await tester.pumpWidget(buildGrid(stations: [
        StationData(name: '起点', x: 0, wl: 3.0, wr: 3.0),
        StationData(name: '終点', x: 100, wl: 3.0, wr: 3.0),
      ]));
      expect(find.text('起点'), findsWidgets);
      expect(find.text('終点'), findsWidgets);
    });
  });

  // ================================================================
  // Cell editing
  // ================================================================

  group('cell editing', () {
    testWidgets('editing station name triggers onChanged', (tester) async {
      List<StationData>? changed;
      await tester.pumpWidget(buildGrid(
        stations: [StationData(name: 'No.0', x: 0, wl: 2.5, wr: 2.5)],
        onChanged: (s) => changed = s,
      ));

      // Find TextFormField with 'No.0'
      final nameField = find.widgetWithText(TextFormField, 'No.0');
      expect(nameField, findsOneWidget);

      await tester.tap(nameField);
      await tester.pump();

      await tester.enterText(nameField, 'BP');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pump();

      expect(changed, isNotNull);
      expect(changed![0].name, 'BP');
    });

    testWidgets('editing numeric field with valid number updates value', (tester) async {
      List<StationData>? changed;
      await tester.pumpWidget(buildGrid(
        stations: [StationData(name: 'No.0', x: 10.0, wl: 2.5, wr: 2.5)],
        onChanged: (s) => changed = s,
      ));

      // Find x field (value '10.0')
      final xField = find.widgetWithText(TextFormField, '10.0');
      expect(xField, findsOneWidget);

      await tester.tap(xField);
      await tester.pump();

      await tester.enterText(xField, '25.5');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pump();

      expect(changed, isNotNull);
      expect(changed![0].x, closeTo(25.5, 1e-9));
    });
  });

  // ================================================================
  // External state update
  // ================================================================

  group('external update', () {
    testWidgets('didUpdateWidget refreshes rows', (tester) async {
      final stations1 = [StationData(name: 'A', x: 0, wl: 1, wr: 1)];
      final stations2 = [
        StationData(name: 'B', x: 0, wl: 1, wr: 1),
        StationData(name: 'C', x: 10, wl: 1, wr: 1),
      ];

      await tester.pumpWidget(buildGrid(stations: stations1));
      var dt = tester.widget<DataTable>(find.byType(DataTable));
      expect(dt.rows.length, 1);

      await tester.pumpWidget(buildGrid(stations: stations2));
      dt = tester.widget<DataTable>(find.byType(DataTable));
      expect(dt.rows.length, 2);
    });
  });
}
