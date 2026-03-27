import 'dart:math' as math;
import 'package:flutter/material.dart';
import '../models/preview_data.dart';

class DxfPreview extends StatefulWidget {
  final PreviewData data;
  const DxfPreview({super.key, required this.data});

  @override
  State<DxfPreview> createState() => _DxfPreviewState();
}

class _DxfPreviewState extends State<DxfPreview> {
  Offset _pan = Offset.zero;
  double _zoom = 1.0;

  double _previousScale = 1.0;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onScaleStart: (d) => _previousScale = _zoom,
      onScaleUpdate: (d) {
        setState(() {
          // Pan (single finger drag)
          _pan += d.focalPointDelta;
          // Zoom (pinch)
          if (d.scale != 1.0) {
            _zoom = (_previousScale * d.scale).clamp(0.01, 100.0);
          }
        });
      },
      child: ClipRect(
        child: CustomPaint(
          painter: _DxfPainter(
            data: widget.data,
            pan: _pan,
            zoom: _zoom,
          ),
          size: Size.infinite,
        ),
      ),
    );
  }
}

class _DxfPainter extends CustomPainter {
  final PreviewData data;
  final Offset pan;
  final double zoom;

  _DxfPainter({required this.data, required this.pan, required this.zoom});

  @override
  void paint(Canvas canvas, Size size) {
    // Dark background
    canvas.drawRect(
      Rect.fromLTWH(0, 0, size.width, size.height),
      Paint()..color = const Color(0xFF1A1A2E),
    );

    if (data.lines.isEmpty && data.texts.isEmpty) return;

    // Calculate bounding box
    double minX = double.infinity, minY = double.infinity;
    double maxX = double.negativeInfinity, maxY = double.negativeInfinity;
    for (final l in data.lines) {
      minX = math.min(minX, math.min(l.x1, l.x2));
      minY = math.min(minY, math.min(l.y1, l.y2));
      maxX = math.max(maxX, math.max(l.x1, l.x2));
      maxY = math.max(maxY, math.max(l.y1, l.y2));
    }
    if (minX == double.infinity) return;

    final dxfW = maxX - minX;
    final dxfH = maxY - minY;
    if (dxfW <= 0 || dxfH <= 0) return;

    // Fit-to-view scale (with 10% margin)
    const margin = 0.9;
    final scaleX = size.width * margin / dxfW;
    final scaleY = size.height * margin / dxfH;
    final baseScale = math.min(scaleX, scaleY) * zoom;

    // Center offset
    final cx = (size.width - dxfW * baseScale) / 2 + pan.dx;
    final cy = (size.height - dxfH * baseScale) / 2 + pan.dy;

    // Transform: DXF Y-up → screen Y-down
    Offset transform(double x, double y) {
      return Offset(
        cx + (x - minX) * baseScale,
        cy + (maxY - y) * baseScale, // flip Y
      );
    }

    // Draw lines
    for (final l in data.lines) {
      final paint = Paint()
        ..color = _dxfColor(l.color)
        ..strokeWidth = 1.0;
      canvas.drawLine(transform(l.x1, l.y1), transform(l.x2, l.y2), paint);
    }

    // Draw texts
    for (final t in data.texts) {
      final fontSize = math.max(8.0, t.height * baseScale * 0.01);
      final textPainter = TextPainter(
        text: TextSpan(
          text: t.text,
          style: TextStyle(color: _dxfColor(t.color), fontSize: fontSize),
        ),
        textDirection: TextDirection.ltr,
      )..layout();

      final pos = transform(t.x, t.y);
      canvas.save();
      canvas.translate(pos.dx, pos.dy);
      if (t.rotation != 0) {
        canvas.rotate(-t.rotation * math.pi / 180); // DXF rotation is CCW
      }
      // Center text at anchor point
      textPainter.paint(canvas, Offset(-textPainter.width / 2, -textPainter.height / 2));
      canvas.restore();
    }
  }

  Color _dxfColor(int dxfColor) {
    const map = {
      1: Color(0xFFFF0000), // red
      2: Color(0xFFFFFF00), // yellow
      3: Color(0xFF00FF00), // green
      4: Color(0xFF00FFFF), // cyan
      5: Color(0xFF0080FF), // blue (matches Rust 0x0080FF)
      6: Color(0xFFFF00FF), // magenta
      7: Color(0xFFFFFFFF), // white
    };
    return map[dxfColor] ?? const Color(0xFFCCCCCC);
  }

  @override
  bool shouldRepaint(_DxfPainter old) =>
      old.data != data || old.pan != pan || old.zoom != zoom;
}
