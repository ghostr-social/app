import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

extension FeedPreviewGestures on WidgetTester {
  Future<void> pumpPreview() async {
    for (var frame = 0; frame < 8; frame++) {
      await pump(const Duration(milliseconds: 20));
      await runAsync(() => Future<void>.delayed(Duration.zero));
    }
  }

  Future<TestGesture> dragPreview(double pages) async {
    final page = find.byType(PageView);
    final gesture = await startGesture(getCenter(page));
    await gesture.moveBy(Offset(0, getSize(page).height * pages));
    await pumpPreview();
    return gesture;
  }
}
