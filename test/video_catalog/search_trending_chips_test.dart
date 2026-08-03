import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/trending_hashtags.dart';

import '../support/fakes.dart';
import '../support/search_screen_harness.dart';

void main() {
  testWidgets('trending tags greet an idle search and open as feeds',
      (tester) async {
    final opened = <String>[];
    final repository = FakeVideoCatalogRepository(forYouFeed: []);
    await tester.pumpWidget(searchScreenHarness(
      repository,
      trending: _StubTrending(const ['dance', 'music']),
      onOpenFeed: opened.add,
    ));
    await tester.pumpAndSettle();

    expect(find.text('Trending now'), findsOneWidget);
    expect(find.text('#dance'), findsOneWidget);
    expect(find.text('#music'), findsOneWidget);

    await tester.tap(find.text('#dance'));
    expect(opened, ['#dance']);
  });
}

class _StubTrending implements TrendingHashtagsSource {
  const _StubTrending(this.tags);

  final List<String> tags;

  @override
  Future<List<String>> trendingHashtags() async => tags;
}
