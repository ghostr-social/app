import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_page_view.dart';

void main() {
  testWidgets('a feed page view starts on the requested page', (tester) async {
    final changes = <int>[];
    await tester.pumpWidget(
      MaterialApp(
        home: FeedPageView(
          model: FeedPageModel(
            keys: List.generate(3, (index) => ValueKey('page-$index')),
            activePage: 2,
          ),
          onPageChanged: (changed) {
            changes.add(changed);
            return true;
          },
          itemBuilder: (_, index) => Text('Page $index'),
        ),
      ),
    );

    expect(find.text('Page 2'), findsOneWidget);
    expect(changes, isEmpty);
  });
}
