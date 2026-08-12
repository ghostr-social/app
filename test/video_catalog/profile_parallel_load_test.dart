import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/aggregating_video_profile_repository.dart';

import '../support/profile_aggregation_probe.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'profile aggregation starts posts and social reads concurrently',
    () async {
      final viewer = sampleSession().profile;
      final target = sampleCreator();
      final probe = ProfileAggregationProbe();
      final repository = AggregatingVideoProfileRepository(probe, probe);
      final loading = repository.loadProfile(viewer, target.id);
      addTearDown(() async {
        probe.release(loadedPosts: [samplePost(creator: target)]);
        await loading;
      });

      await Future<void>.delayed(Duration.zero);

      expect(probe.postReads, 1);
      expect(probe.followedReads, 1);
      expect(probe.blockedReads, 1);
    },
  );
}
