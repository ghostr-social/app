import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/following_feed_scope.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import 'nostr_test_values.dart';

FollowingFeedScopeReader testFollowingFeedScopes(
  SocialGraphRepository social, {
  FollowingFeedViewer? viewer,
}) {
  return FollowingFeedScopeReader(
    social,
    viewer ?? () => NostrPublicKeyHex.parse(testViewerPublicKey),
  );
}

FollowingFeedScope testFollowingFeedScope({Set<ProfileId>? creators}) {
  return FollowingFeedScope(
    viewer: NostrPublicKeyHex.parse(testViewerPublicKey),
    creators: creators ?? <ProfileId>{ProfileId.parse(testCreatorNpub)},
  );
}
