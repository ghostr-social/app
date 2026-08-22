import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_page_view.dart';

void main() {
  testWidgets('a rejected page commit returns to the active page', (
    tester,
  ) async {
    final changes = <int>[];
    await tester.pumpWidget(
      MaterialApp(
        home: FeedPageView(
          model: FeedPageModel(
            keys: const [ValueKey('page-0'), ValueKey('page-1')],
          ),
          onPageChanged: (index) {
            changes.add(index);
            return false;
          },
          itemBuilder: (_, index) => Text('Page $index'),
        ),
      ),
    );

    await tester.drag(find.byType(PageView), const Offset(0, -600));
    await tester.pumpAndSettle();

    expect(changes, [1]);
    expect(find.text('Page 0'), findsOneWidget);
  });
}
