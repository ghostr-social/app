import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/social/domain/nostr_social_port.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/memory_social_graph_store.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';

void main() {
  test('mute reads share one request until freshness expires', () async {
    final remote = _DeferredBlockedSocial();
    final local = MemorySocialGraphStore(
      accountPublicKey: remote.accountPublicKey,
    );
    var now = DateTime.utc(2026);
    final graph = SocialGraphCache(
      remote,
      local,
      RecordingFailureReporter(),
      clock: () => now,
    );
    final blocked = ProfileId.parse('blocked');

    final first = graph.loadBlockedProfiles();
    final second = graph.loadBlockedProfiles();
    await remote.started.future;
    remote.result.complete({blocked});

    expect(await Future.wait([first, second]), [
      {blocked},
      {blocked},
    ]);
    expect(await graph.loadBlockedProfiles(), {blocked});
    expect(remote.calls, 1);

    now = now.add(const Duration(minutes: 2));
    expect(await graph.loadBlockedProfiles(), {blocked});
    expect(remote.calls, 2);
  });
}

final class _DeferredBlockedSocial implements NostrSocialPort {
  final started = Completer<void>();
  final result = Completer<Set<ProfileId>>();
  var calls = 0;

  @override
  NostrPublicKeyHex get accountPublicKey =>
      NostrPublicKeyHex.parse(testViewerPublicKey);

  @override
  NostrSocialPort snapshotForActiveAccount() => this;

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() {
    calls += 1;
    if (!started.isCompleted) started.complete();
    return result.future;
  }

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async => const <ProfileId>{};

  @override
  Future<bool> toggleBlock(ProfileId profileId) async => false;

  @override
  Future<bool> toggleFollow(ProfileId profileId) async => false;
}
