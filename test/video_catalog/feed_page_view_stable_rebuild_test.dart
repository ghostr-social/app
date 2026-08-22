import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_page_view.dart';

void main() {
  testWidgets('an unchanged active page does not reset an in-progress swipe', (
    tester,
  ) async {
    final rebuild = ValueNotifier(0);
    addTearDown(rebuild.dispose);
    await tester.pumpWidget(
      MaterialApp(
        home: ValueListenableBuilder<int>(
          valueListenable: rebuild,
          builder: (_, value, __) => FeedPageView(
            model: FeedPageModel(
              keys: const [ValueKey('page-0'), ValueKey('page-1')],
            ),
            onPageChanged: (_) => true,
            itemBuilder: (_, index) => Text('Page $index rebuild $value'),
          ),
        ),
      ),
    );
    final scrollable = tester.state<ScrollableState>(find.byType(Scrollable));
    final gesture = await tester.startGesture(
      tester.getCenter(find.byType(FeedPageView)),
    );
    await gesture.moveBy(const Offset(0, -500));
    await tester.pump();
    final draggedPixels = scrollable.position.pixels;
    expect(draggedPixels, greaterThan(0));

    rebuild.value += 1;
    await tester.pump();

    expect(scrollable.position.pixels, closeTo(draggedPixels, 1));
    await gesture.cancel();
  });
}
