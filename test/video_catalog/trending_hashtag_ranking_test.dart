import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/trending_hashtags.dart';

import '../support/sample_data.dart';

void main() {
  test('hashtags rank by recent usage with stable alphabetic ties', () {
    final posts = [
      samplePost(id: 'a', hashtags: const ['dance', 'music']),
      samplePost(id: 'b', hashtags: const ['dance', 'art']),
      samplePost(id: 'c', hashtags: const ['dance', 'music']),
      samplePost(id: 'd', hashtags: const ['zebra']),
    ];

    expect(
      rankTrendingHashtags(posts),
      ['dance', 'music', 'art', 'zebra'],
    );
    expect(rankTrendingHashtags(posts, limit: 2), ['dance', 'music']);
    expect(rankTrendingHashtags(const []), isEmpty);
  });
}
