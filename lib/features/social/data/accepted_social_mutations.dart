import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

enum SocialGraphMembership { followed, blocked }

typedef _MutationKey = (NostrPublicKeyHex, SocialGraphMembership);

final class AcceptedSocialMutations {
  final _pending = <_MutationKey, Map<ProfileId, bool>>{};

  void accept(
    NostrPublicKeyHex account,
    SocialGraphMembership membership,
    ProfileId profile,
    bool included,
  ) {
    final desired = _pending.putIfAbsent((account, membership), () => {});
    desired[profile] = included;
  }

  Set<ProfileId> project(
    NostrPublicKeyHex account,
    SocialGraphMembership membership,
    Set<ProfileId> source, {
    bool observed = false,
  }) {
    final key = (account, membership);
    final desired = _pending[key];
    if (desired == null) return {...source};
    if (observed) _forgetObserved(desired, source);
    final projected = _apply(source, desired);
    if (desired.isEmpty) _pending.remove(key);
    return projected;
  }

  void _forgetObserved(
    Map<ProfileId, bool> desired,
    Set<ProfileId> observed,
  ) {
    final confirmed = desired.keys.where((profile) {
      return observed.contains(profile) == desired[profile];
    }).toList(growable: false);
    for (final profile in confirmed) {
      desired.remove(profile);
    }
  }

  Set<ProfileId> _apply(
    Set<ProfileId> source,
    Map<ProfileId, bool> desired,
  ) {
    final projected = {...source};
    for (final entry in desired.entries) {
      entry.value ? projected.add(entry.key) : projected.remove(entry.key);
    }
    return projected;
  }
}
