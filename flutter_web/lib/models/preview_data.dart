import 'dart:convert';

class PreviewLine {
  final double x1, y1, x2, y2;
  final int color;
  PreviewLine({required this.x1, required this.y1, required this.x2, required this.y2, required this.color});

  factory PreviewLine.fromJson(Map<String, dynamic> j) => PreviewLine(
    x1: (j['x1'] as num).toDouble(), y1: (j['y1'] as num).toDouble(),
    x2: (j['x2'] as num).toDouble(), y2: (j['y2'] as num).toDouble(),
    color: j['color'] as int? ?? 7,
  );
}

class PreviewText {
  final String text;
  final double x, y, rotation, height;
  final int color;
  PreviewText({required this.text, required this.x, required this.y,
    required this.rotation, required this.height, required this.color});

  factory PreviewText.fromJson(Map<String, dynamic> j) => PreviewText(
    text: j['text'] as String? ?? '',
    x: (j['x'] as num).toDouble(), y: (j['y'] as num).toDouble(),
    rotation: (j['rotation'] as num?)?.toDouble() ?? 0,
    height: (j['height'] as num?)?.toDouble() ?? 350,
    color: j['color'] as int? ?? 7,
  );
}

class PreviewData {
  final List<PreviewLine> lines;
  final List<PreviewText> texts;
  PreviewData({required this.lines, required this.texts});

  factory PreviewData.empty() => PreviewData(lines: [], texts: []);

  factory PreviewData.fromJson(String jsonStr) {
    final map = jsonDecode(jsonStr) as Map<String, dynamic>;
    return PreviewData(
      lines: (map['lines'] as List).map((e) => PreviewLine.fromJson(e as Map<String, dynamic>)).toList(),
      texts: (map['texts'] as List).map((e) => PreviewText.fromJson(e as Map<String, dynamic>)).toList(),
    );
  }
}
