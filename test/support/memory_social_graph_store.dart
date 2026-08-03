import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/domain/social_graph_store.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

final class MemorySocialGraphStore implements SocialGraphStore {
  MemorySocialGraphStore({
    required this.accountPublicKey,
    Set<ProfileId>? followed,
    Set<ProfileId>? blocked,
    this.rejectFollowWrites = false,
    this.rejectBlockWrites = false,
  })  : followed = followed ?? <ProfileId>{},
        blocked = blocked ?? <ProfileId>{};

  @override
  final NostrPublicKeyHex accountPublicKey;
  Set<ProfileId> followed;
  Set<ProfileId> blocked;
  bool rejectFollowWrites;
  bool rejectBlockWrites;

  @override
  SocialGraphStore snapshotForActiveAccount() => this;

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async => {...followed};

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async => {...blocked};

  @override
  Future<void> saveFollowedProfiles(Set<ProfileId> profileIds) async {
    if (rejectFollowWrites) throw StateError('follow cache unavailable');
    followed = {...profileIds};
  }

  @override
  Future<void> saveBlockedProfiles(Set<ProfileId> profileIds) async {
    if (rejectBlockWrites) throw StateError('block cache unavailable');
    blocked = {...profileIds};
  }
}
