final class VideoShareOrigin {
  const VideoShareOrigin({
    required this.left,
    required this.top,
    required this.width,
    required this.height,
  });

  static const zero = VideoShareOrigin(left: 0, top: 0, width: 0, height: 0);

  final double left;
  final double top;
  final double width;
  final double height;

  @override
  bool operator ==(Object other) {
    return other is VideoShareOrigin &&
        other.left == left &&
        other.top == top &&
        other.width == width &&
        other.height == height;
  }

  @override
  int get hashCode => Object.hash(left, top, width, height);
}
