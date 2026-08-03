import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/video_catalog/data/local_video_store.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_social_port.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';

void main() {
  test('a completed social mutation caches only for its initiating account',
      () async {
    SharedPreferences.setMockInitialValues({});
    var account = NostrPublicKeyHex.parse(testViewerPublicKey);
    final local = LocalVideoStore(
      await SharedPreferences.getInstance(),
      accountScope: AccountStorageScope(() => account),
    );
    final remote = _DelayedSocialPort();
    final cache = SocialGraphCache(remote, local, RecordingFailureReporter());
    final creator = ProfileId.parse('creator');

    final pending = cache.toggleFollow(creator);
    await remote.started.future;
    account = NostrPublicKeyHex.parse(testAuthorPublicKey);
    remote.release.complete();
    await pending;

    expect(await local.loadFollowedProfiles(), isEmpty);
    account = NostrPublicKeyHex.parse(testViewerPublicKey);
    expect(await local.loadFollowedProfiles(), {creator});
  });
}

class _DelayedSocialPort extends FakeNostrSocialPort {
  final started = Completer<void>();
  final release = Completer<void>();

  @override
  Future<bool> toggleFollow(ProfileId profileId) async {
    started.complete();
    await release.future;
    return super.toggleFollow(profileId);
  }
}
