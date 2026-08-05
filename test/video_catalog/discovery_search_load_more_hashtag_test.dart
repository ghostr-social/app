import 'package:flutter_test/flutter_test.dart';

import '../support/discovery_search_harness.dart';
import '../support/sample_data.dart';

void main() {
  test('hashtag load-more uses the native command and rechecks tags', () async {
    final harness = DiscoverySearchHarness(posts: [
      samplePost(id: 'tagged', hashtags: const ['dance']),
      samplePost(id: 'other', hashtags: const ['music']),
    ]);

    final page = await harness.repository.loadMoreVideos(' #Dance ');

    expect(harness.source.loadMoreQueries, [null]);
    expect(harness.source.loadMoreHashtags, [
      {'dance'},
    ]);
    expect(page.posts.single.id.value, 'tagged');
  });
}
