import 'package:flutter/widgets.dart';

FixedScrollMetrics feedSwipeMetrics({
  double pixels = 500,
  double minScrollExtent = 0,
  double maxScrollExtent = 1500,
  double viewportDimension = 500,
}) {
  return FixedScrollMetrics(
    minScrollExtent: minScrollExtent,
    maxScrollExtent: maxScrollExtent,
    pixels: pixels,
    viewportDimension: viewportDimension,
    axisDirection: AxisDirection.down,
    devicePixelRatio: 1,
  );
}
