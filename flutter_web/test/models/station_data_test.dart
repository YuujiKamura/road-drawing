import 'package:flutter_test/flutter_test.dart';
import 'package:road_drawing_flutter/models/station_data.dart';

void main() {
  // ================================================================
  // toCsv
  // ================================================================

  group('toCsv', () {
    test('generates valid CSV with header', () {
      final stations = [
        StationData(name: 'No.0', x: 0, wl: 3.45, wr: 3.55),
        StationData(name: 'No.1', x: 20, wl: 3.50, wr: 3.50),
      ];
      final csv = StationData.toCsv(stations);
      expect(csv, startsWith('name,x,wl,wr\n'));
      expect(csv, contains('No.0,0.0,3.45,3.55'));
      expect(csv, contains('No.1,20.0,3.5,3.5'));
    });

    test('empty list produces header only', () {
      final csv = StationData.toCsv([]);
      expect(csv, 'name,x,wl,wr\n');
    });

    test('preserves station names with special chars', () {
      final stations = [
        StationData(name: '0+5.5', x: 5.5, wl: 1, wr: 1),
        StationData(name: 'No.1', x: 20, wl: 1, wr: 1),
      ];
      final csv = StationData.toCsv(stations);
      expect(csv, contains('0+5.5'));
      expect(csv, contains('No.1'));
    });

    test('handles Japanese station names', () {
      final stations = [
        StationData(name: '起点', x: 0, wl: 3.0, wr: 3.0),
        StationData(name: '終点', x: 100, wl: 3.0, wr: 3.0),
      ];
      final csv = StationData.toCsv(stations);
      expect(csv, contains('起点'));
      expect(csv, contains('終点'));
    });

    test('negative and zero widths', () {
      final csv = StationData.toCsv([
        StationData(name: 'A', x: 0, wl: -2.5, wr: 0),
      ]);
      expect(csv, contains('A,0.0,-2.5,0.0'));
    });
  });

  // ================================================================
  // fromJsonList (WASM parse_csv output)
  // ================================================================

  group('fromJsonList', () {
    test('parses single station', () {
      const json = '[{"name":"No.0","x":0.0,"wl":3.45,"wr":3.55}]';
      final stations = StationData.fromJsonList(json);
      expect(stations.length, 1);
      expect(stations[0].name, 'No.0');
      expect(stations[0].x, closeTo(0.0, 1e-9));
      expect(stations[0].wl, closeTo(3.45, 1e-9));
      expect(stations[0].wr, closeTo(3.55, 1e-9));
    });

    test('parses multiple stations', () {
      const json =
          '[{"name":"No.0","x":0,"wl":3.45,"wr":3.55},'
          '{"name":"No.1","x":20,"wl":3.5,"wr":3.5}]';
      final stations = StationData.fromJsonList(json);
      expect(stations.length, 2);
      expect(stations[1].name, 'No.1');
      expect(stations[1].x, closeTo(20.0, 1e-9));
    });

    test('parses empty array', () {
      final stations = StationData.fromJsonList('[]');
      expect(stations, isEmpty);
    });

    test('handles missing fields with defaults', () {
      const json = '[{"name":"A"}]';
      final stations = StationData.fromJsonList(json);
      expect(stations[0].name, 'A');
      expect(stations[0].x, 0.0);
      expect(stations[0].wl, 0.0);
      expect(stations[0].wr, 0.0);
    });

    test('handles Japanese names in JSON', () {
      const json = '[{"name":"起点","x":0,"wl":3,"wr":3}]';
      final stations = StationData.fromJsonList(json);
      expect(stations[0].name, '起点');
    });
  });

  // ================================================================
  // fromJson / toJson roundtrip
  // ================================================================

  group('JSON roundtrip', () {
    test('toJson → fromJson preserves all fields', () {
      final original = StationData(name: 'No.0', x: 5.5, wl: 2.5, wr: 3.5);
      final json = original.toJson();
      final restored = StationData.fromJson(json);
      expect(restored.name, original.name);
      expect(restored.x, closeTo(original.x, 1e-9));
      expect(restored.wl, closeTo(original.wl, 1e-9));
      expect(restored.wr, closeTo(original.wr, 1e-9));
    });

    test('roundtrip with zero values', () {
      final original = StationData(name: '', x: 0, wl: 0, wr: 0);
      final json = original.toJson();
      final restored = StationData.fromJson(json);
      expect(restored.name, '');
      expect(restored.x, 0.0);
    });
  });
}
