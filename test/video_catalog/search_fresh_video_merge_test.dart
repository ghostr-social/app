import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/search_state.dart';

import '../support/sample_data.dart';

void main() {
  test('fresh videos lead while preserving prior rows and page availability',
      () {
    final older = samplePost(id: 'older');
    final fresh = samplePost(id: 'fresh');
    final state = SearchLoaded(
      'ghost',
      SearchResults(videos: [older], hasMore: true),
      isLoadingMore: true,
      notice: 'still searching',
    );

    final updated = state.withFreshVideos([fresh, older], false);

    expect(updated.query, 'ghost');
    expect(updated.videos, [fresh, older]);
    expect(updated.hasMore, isTrue);
    expect(updated.isLoadingMore, isTrue);
    expect(updated.notice, 'still searching');
  });
}
