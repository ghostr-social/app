import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/aggregating_video_profile_repository.dart';

import '../support/profile_aggregation_probe.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'current-user profile aggregation skips the irrelevant blocked read',
    () async {
      final viewer = sampleSession().profile;
      final post = samplePost(creator: viewer);
      final probe = ProfileAggregationProbe()
        ..posts.complete([post])
        ..followed.complete(const {})
        ..blocked.complete({viewer.id});
      final repository = AggregatingVideoProfileRepository(probe, probe);

      final details = await repository.loadProfile(viewer, viewer.id);

      expect(probe.blockedReads, 0);
      expect(details.profile, same(viewer));
      expect(details.isCurrentUser, isTrue);
    },
  );
}
