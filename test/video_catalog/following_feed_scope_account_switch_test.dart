import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/following_feed_scope.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fake_social_graph_repository.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('account switch rejects the captured follow set', () async {
    final social = _GatedSocial();
    var viewer = NostrPublicKeyHex.parse(testViewerPublicKey);
    final scopes = FollowingFeedScopeReader(social, () => viewer);

    final reading = scopes.load();
    await social.started.future;
    viewer = NostrPublicKeyHex.parse(testCreatorPublicKey);
    social.release.complete();

    await expectLater(reading, throwsA(isA<AppFailure>()));
  });
}

final class _GatedSocial extends FakeSocialGraphRepository {
  final started = Completer<void>();
  final release = Completer<void>();

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async {
    started.complete();
    await release.future;
    return {ProfileId.parse(testViewerNpub)};
  }
}
