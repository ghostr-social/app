import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/video_catalog/data/local_video_store.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fakes.dart';

void main() {
  test('mirrors successful relay social changes into the local cache',
      () async {
    SharedPreferences.setMockInitialValues({});
    final local = LocalVideoStore(await SharedPreferences.getInstance());
    final cache = SocialGraphCache(
      FakeNostrSocialPort(),
      local,
      RecordingFailureReporter(),
    );
    final creator = ProfileId.parse('creator');

    expect(await cache.toggleFollow(creator), isTrue);
    expect(await cache.toggleBlock(creator), isTrue);
    expect(await local.loadFollowedProfiles(), {creator});
    expect(await local.loadBlockedProfiles(), {creator});

    expect(await cache.toggleFollow(creator), isFalse);
    expect(await cache.toggleBlock(creator), isFalse);
    expect(await local.loadFollowedProfiles(), isEmpty);
    expect(await local.loadBlockedProfiles(), isEmpty);
  });
}
