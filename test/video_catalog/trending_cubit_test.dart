import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/trending_hashtags.dart';
import 'package:ghostr/features/video_catalog/presentation/trending_hashtags_cubit.dart';

void main() {
  test('trending is ready with tags and silently unavailable otherwise',
      () async {
    final ready = TrendingHashtagsCubit(_StubTrending(['dance', 'music']));
    addTearDown(ready.close);
    await ready.load();
    expect(
      (ready.state as TrendingHashtagsReady).tags,
      ['dance', 'music'],
    );

    final empty = TrendingHashtagsCubit(_StubTrending(const []));
    addTearDown(empty.close);
    await empty.load();
    expect(empty.state, isA<TrendingHashtagsUnavailable>());

    final failing = TrendingHashtagsCubit(_StubTrending(null));
    addTearDown(failing.close);
    await failing.load();
    expect(failing.state, isA<TrendingHashtagsUnavailable>());
  });
}

class _StubTrending implements TrendingHashtagsSource {
  _StubTrending(this.tags);

  final List<String>? tags;

  @override
  Future<List<String>> trendingHashtags() async {
    final available = tags;
    if (available == null) throw StateError('offline');
    return available;
  }
}
