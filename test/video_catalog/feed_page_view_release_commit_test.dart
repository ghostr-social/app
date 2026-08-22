import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_page_view.dart';

void main() {
  testWidgets('a page change commits once when the pointer is released', (
    tester,
  ) async {
    final pages = ValueNotifier(['p0', 'p1', 'p2']);
    final committed = <String>[];
    addTearDown(pages.dispose);
    await tester.pumpWidget(
      MaterialApp(
        home: ValueListenableBuilder<List<String>>(
          valueListenable: pages,
          builder: (_, value, __) => FeedPageView(
            model: FeedPageModel(keys: value.map((page) => ValueKey(page))),
            itemBuilder: (_, index) => Text(value[index]),
            onPageChanged: (index) {
              committed.add(value[index]);
              pages.value = value.sublist(index);
              return true;
            },
          ),
        ),
      ),
    );
    final view = find.byType(PageView);
    final gesture = await tester.startGesture(tester.getCenter(view));
    await gesture.moveBy(Offset(0, -tester.getSize(view).height * 0.23));
    await tester.pump(const Duration(milliseconds: 16));
    expect(committed, isEmpty);

    await gesture.up();
    await tester.pumpAndSettle();
    expect(committed, ['p1']);
  });
}
