import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/video_catalog/data/local_video_store.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fakes.dart';

void main() {
  test('does not fabricate cached social changes after a relay failure',
      () async {
    SharedPreferences.setMockInitialValues({});
    final local = LocalVideoStore(await SharedPreferences.getInstance());
    final remote = FakeNostrSocialPort()
      ..toggleFailure = const AppFailure('relay rejected write');
    final cache = SocialGraphCache(
      remote,
      local,
      RecordingFailureReporter(),
    );
    final creator = ProfileId.parse('creator');

    await expectLater(cache.toggleFollow(creator), throwsA(isA<AppFailure>()));
    await expectLater(cache.toggleBlock(creator), throwsA(isA<AppFailure>()));
    expect(await local.loadFollowedProfiles(), isEmpty);
    expect(await local.loadBlockedProfiles(), isEmpty);
  });
}
