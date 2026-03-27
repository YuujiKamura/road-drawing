import '../models/preview_data.dart';
import '../models/station_data.dart';

/// Pure Dart fallback for preview data generation.
/// Used when WASM bridge is not yet available.
/// Mirrors the Rust calculate_road_section logic.
class PreviewFallback {
  static const double scale = 1000.0; // m → mm

  static PreviewData calculate(List<StationData> stations) {
    if (stations.isEmpty) return PreviewData.empty();

    final lines = <PreviewLine>[];
    final texts = <PreviewText>[];

    for (int i = 0; i < stations.length; i++) {
      final s = stations[i];
      final cx = s.x * scale;
      final wlMm = s.wl * scale;
      final wrMm = s.wr * scale;

      // Width lines: center → left (positive Y), center → right (negative Y)
      lines.add(PreviewLine(x1: cx, y1: 0, x2: cx, y2: wlMm, color: 7));
      lines.add(PreviewLine(x1: cx, y1: 0, x2: cx, y2: -wrMm, color: 7));

      // Station name label (blue, color=5)
      texts.add(PreviewText(
        text: s.name, x: cx, y: -wrMm - 500,
        rotation: 0, height: 350, color: 5,
      ));

      // Width dimension texts (rotated -90°)
      if (s.wl > 0) {
        texts.add(PreviewText(
          text: s.wl.toStringAsFixed(2),
          x: cx - 200, y: wlMm / 2,
          rotation: -90, height: 250, color: 3,
        ));
      }
      if (s.wr > 0) {
        texts.add(PreviewText(
          text: s.wr.toStringAsFixed(2),
          x: cx - 200, y: -wrMm / 2,
          rotation: -90, height: 250, color: 3,
        ));
      }

      // Connecting lines to next station
      if (i < stations.length - 1) {
        final next = stations[i + 1];
        final ncx = next.x * scale;
        final nwl = next.wl * scale;
        final nwr = next.wr * scale;

        // Center line
        lines.add(PreviewLine(x1: cx, y1: 0, x2: ncx, y2: 0, color: 7));
        // Top outline (left widths)
        if (s.wl > 0 || next.wl > 0) {
          lines.add(PreviewLine(x1: cx, y1: wlMm, x2: ncx, y2: nwl, color: 7));
        }
        // Bottom outline (right widths)
        if (s.wr > 0 || next.wr > 0) {
          lines.add(PreviewLine(x1: cx, y1: -wrMm, x2: ncx, y2: -nwr, color: 7));
        }

        // Distance dimension
        final dist = (next.x - s.x).abs();
        texts.add(PreviewText(
          text: dist.toStringAsFixed(2),
          x: (cx + ncx) / 2, y: 300,
          rotation: 0, height: 250, color: 1,
        ));
      }
    }

    return PreviewData(lines: lines, texts: texts);
  }
}
