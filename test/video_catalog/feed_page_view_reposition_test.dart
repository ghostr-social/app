import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_page_view.dart';

void main() {
  testWidgets('a changed active page repositions the visible feed', (
    tester,
  ) async {
    final page = ValueNotifier(0);
    final changes = <int>[];
    addTearDown(page.dispose);
    await tester.pumpWidget(
      MaterialApp(
        home: ValueListenableBuilder<int>(
          valueListenable: page,
          builder: (_, index, __) => FeedPageView(
            model: FeedPageModel(
              keys: List.generate(3, (page) => ValueKey('page-$page')),
              activePage: index,
            ),
            onPageChanged: (changed) {
              changes.add(changed);
              return true;
            },
            itemBuilder: (_, item) => Text('Page $item'),
          ),
        ),
      ),
    );

    page.value = 2;
    await tester.pumpAndSettle();

    expect(find.text('Page 2'), findsOneWidget);
    expect(changes.last, 2);
  });
}
