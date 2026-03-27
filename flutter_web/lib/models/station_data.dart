import 'dart:convert';

class StationData {
  String name;
  double x;
  double wl;
  double wr;

  StationData({
    required this.name,
    required this.x,
    required this.wl,
    required this.wr,
  });

  factory StationData.fromJson(Map<String, dynamic> json) {
    return StationData(
      name: json['name'] as String? ?? '',
      x: (json['x'] as num?)?.toDouble() ?? 0.0,
      wl: (json['wl'] as num?)?.toDouble() ?? 0.0,
      wr: (json['wr'] as num?)?.toDouble() ?? 0.0,
    );
  }

  Map<String, dynamic> toJson() => {'name': name, 'x': x, 'wl': wl, 'wr': wr};

  /// Convert list of StationData to CSV text (for WASM input)
  static String toCsv(List<StationData> stations) {
    final buf = StringBuffer('name,x,wl,wr\n');
    for (final s in stations) {
      buf.writeln('${s.name},${s.x},${s.wl},${s.wr}');
    }
    return buf.toString();
  }

  /// Parse JSON array string from WASM parse_csv output
  static List<StationData> fromJsonList(String jsonStr) {
    final list = jsonDecode(jsonStr) as List;
    return list.map((e) => StationData.fromJson(e as Map<String, dynamic>)).toList();
  }

  /// Parse CSV text into station list (Dart fallback, no WASM needed).
  /// Handles header detection, full-width digits, empty lines.
  static List<StationData> fromCsv(String csv) {
    final rows = <StationData>[];
    for (final rawLine in csv.split('\n')) {
      final line = rawLine.trim();
      if (line.isEmpty) continue;

      final parts = line.split(',').map((s) => s.trim()).toList();
      if (parts.length < 2) continue;

      // Skip headers
      final first = parts[0].toLowerCase();
      if (first.contains('測点') ||
          first.contains('name') ||
          first.contains('station') ||
          first.contains('延長')) {
        continue;
      }

      final xVal = double.tryParse(_normalizeNumber(parts[1]));
      if (xVal == null) continue;

      rows.add(StationData(
        name: parts[0],
        x: xVal,
        wl: double.tryParse(_normalizeNumber(parts.length > 2 ? parts[2] : '0')) ?? 0,
        wr: double.tryParse(_normalizeNumber(parts.length > 3 ? parts[3] : '0')) ?? 0,
      ));
    }
    return rows;
  }

  /// Normalize full-width digits to ASCII.
  static String _normalizeNumber(String s) {
    const fullWidth = '０１２３４５６７８９．';
    const halfWidth = '0123456789.';
    final buf = StringBuffer();
    for (final c in s.runes) {
      final ch = String.fromCharCode(c);
      final idx = fullWidth.indexOf(ch);
      if (idx >= 0) {
        buf.write(halfWidth[idx]);
      } else if (ch == '－' || ch == 'ー') {
        buf.write('-');
      } else {
        buf.write(ch);
      }
    }
    return buf.toString();
  }
}
