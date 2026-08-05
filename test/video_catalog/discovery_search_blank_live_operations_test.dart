import 'package:flutter_test/flutter_test.dart';

import '../support/discovery_search_harness.dart';

void main() {
  test('blank live search operations never reach the remote source', () async {
    final harness = DiscoverySearchHarness();

    final page = await harness.repository.loadMoreVideos('   ');
    await expectLater(harness.repository.watchVideos('   '), emitsDone);

    expect(page.posts, isEmpty);
    expect(page.hasMore, isFalse);
    expect(harness.source.loadMoreQueries, isEmpty);
    expect(harness.source.watchQueries, isEmpty);
  });
}
