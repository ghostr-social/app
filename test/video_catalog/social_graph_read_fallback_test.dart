import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/video_catalog/data/local_video_store.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fakes.dart';

void main() {
  test('uses cached social lists when relay reads fail', () async {
    SharedPreferences.setMockInitialValues({});
    final local = LocalVideoStore(await SharedPreferences.getInstance());
    final followed = ProfileId.parse('followed');
    final blocked = ProfileId.parse('blocked');
    await local.saveFollowedProfiles({followed});
    await local.saveBlockedProfiles({blocked});
    final remote = FakeNostrSocialPort()
      ..loadFailure = const AppFailure('offline');
    final reporter = RecordingFailureReporter();
    final cache = SocialGraphCache(remote, local, reporter);

    expect(await cache.loadFollowedProfiles(), {followed});
    expect(await cache.loadBlockedProfiles(), {blocked});
    expect(reporter.sources, [
      'SocialGraphCache.loadFollowedProfiles',
      'SocialGraphCache.loadBlockedProfiles',
    ]);
  });
}
