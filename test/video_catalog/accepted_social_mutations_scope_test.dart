import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/data/accepted_social_mutations.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('keeps pending mutations isolated and retires observed intent', () {
    final mutations = AcceptedSocialMutations();
    final accountA = NostrPublicKeyHex.parse(testViewerPublicKey);
    final accountB = NostrPublicKeyHex.parse(testCreatorPublicKey);
    final profile = ProfileId.parse('creator');

    mutations.accept(
      accountA,
      SocialGraphMembership.followed,
      profile,
      true,
    );

    expect(
      mutations.project(accountB, SocialGraphMembership.followed, {}),
      isEmpty,
    );
    expect(
      mutations.project(accountA, SocialGraphMembership.blocked, {}),
      isEmpty,
    );
    expect(
      mutations.project(accountA, SocialGraphMembership.followed, {}),
      {profile},
    );
    expect(
      mutations.project(
        accountA,
        SocialGraphMembership.followed,
        {},
        observed: true,
      ),
      {profile},
    );
    expect(
      mutations.project(
        accountA,
        SocialGraphMembership.followed,
        {profile},
        observed: true,
      ),
      {profile},
    );
    expect(
      mutations.project(accountA, SocialGraphMembership.followed, {}),
      isEmpty,
    );
  });
}
