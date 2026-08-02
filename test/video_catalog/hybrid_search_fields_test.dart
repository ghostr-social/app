import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/sample_data.dart';

void main() {
  test('matches hybrid search results by song, creator name, and handle',
      () async {
    final creator = ProfileSummary(
      id: ProfileId.parse('search-creator'),
      displayName: 'Visible Name',
      handle: '@handle-only',
      avatarUrl: null,
    );
    final post = samplePost(creator: creator);
    final harness = await buildHybridRepositoryHarness(
      FakeRemoteVideoSource(<VideoPost>[post]),
    );

    expect(await harness.search.search('original sound'), [post]);
    expect(await harness.search.search('visible name'), [post]);
    expect(await harness.search.search('handle-only'), [post]);
  });
}
