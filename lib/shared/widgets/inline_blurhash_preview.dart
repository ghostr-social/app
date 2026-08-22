import 'package:flutter/material.dart';
import 'package:ghostr/core/media/inline_blurhash.dart';

const _previewRasterWidth = 16;
const _previewRasterHeight = 16;

typedef _PreviewRasterKey = ({String encoded, int width, int height});

/// Decorative, bounded raster rendering of an already-inline BlurHash.
class InlineBlurHashPreview extends StatefulWidget {
  const InlineBlurHashPreview({required this.descriptor, super.key});

  final InlineBlurHash descriptor;

  @override
  State<InlineBlurHashPreview> createState() => _InlineBlurHashPreviewState();
}

class _InlineBlurHashPreviewState extends State<InlineBlurHashPreview> {
  late _PreviewRasterKey _key;
  late List<int> _pixels;

  @override
  void initState() {
    super.initState();
    _decode();
  }

  @override
  void didUpdateWidget(covariant InlineBlurHashPreview oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (_rasterKey(widget.descriptor) != _key) _decode();
  }

  @override
  Widget build(BuildContext context) {
    return ExcludeSemantics(
      child: RepaintBoundary(
        child: CustomPaint(
          painter: _BlurHashPainter(
            pixels: _pixels,
            width: _key.width,
            height: _key.height,
          ),
        ),
      ),
    );
  }

  void _decode() {
    _key = _rasterKey(widget.descriptor);
    _pixels = widget.descriptor.decodeArgb(
      width: _key.width,
      height: _key.height,
    );
  }
}

_PreviewRasterKey _rasterKey(InlineBlurHash descriptor) => (
  encoded: descriptor.encoded,
  width: _previewRasterWidth,
  height: _previewRasterHeight,
);

final class _BlurHashPainter extends CustomPainter {
  const _BlurHashPainter({
    required this.pixels,
    required this.width,
    required this.height,
  });

  final List<int> pixels;
  final int width;
  final int height;

  @override
  void paint(Canvas canvas, Size size) {
    final pixelWidth = size.width / width;
    final pixelHeight = size.height / height;
    final paint = Paint();
    for (var index = 0; index < pixels.length; index += 1) {
      final x = index % width;
      final y = index ~/ width;
      paint.color = Color(pixels[index]);
      canvas.drawRect(
        Rect.fromLTWH(x * pixelWidth, y * pixelHeight, pixelWidth, pixelHeight),
        paint,
      );
    }
  }

  @override
  bool shouldRepaint(covariant _BlurHashPainter oldDelegate) {
    return oldDelegate.pixels != pixels ||
        oldDelegate.width != width ||
        oldDelegate.height != height;
  }
}
