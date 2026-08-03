import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

extension type const NostrReactionTarget._(String value) implements String {
  factory NostrReactionTarget.fromReference(NostrEventReference reference) {
    final identifier = reference.identifier;
    if (identifier == null) {
      return NostrReactionTarget._('e:${reference.eventId}');
    }
    return NostrReactionTarget._(
      'a:${reference.kind}:${reference.authorPublicKeyHex}:$identifier',
    );
  }
}

class NostrLikeMutationKey {
  const NostrLikeMutationKey(this.viewer, this.target);

  final NostrPublicKeyHex viewer;
  final NostrReactionTarget target;

  @override
  bool operator ==(Object other) {
    return other is NostrLikeMutationKey &&
        other.viewer == viewer &&
        other.target == target;
  }

  @override
  int get hashCode => Object.hash(viewer, target);
}
