import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

extension type const NostrRepostTarget._(String value) implements String {
  factory NostrRepostTarget.fromReference(NostrEventReference reference) {
    final identifier = reference.coordinateIdentifier;
    if (identifier == null) {
      return NostrRepostTarget._('e:${reference.eventId}');
    }
    return NostrRepostTarget._(
      'a:${reference.kind}:${reference.authorPublicKeyHex}:$identifier',
    );
  }
}

final class NostrRepostMutationKey {
  const NostrRepostMutationKey(this.viewer, this.target);

  final NostrPublicKeyHex viewer;
  final NostrRepostTarget target;

  @override
  bool operator ==(Object other) {
    return other is NostrRepostMutationKey &&
        other.viewer == viewer &&
        other.target == target;
  }

  @override
  int get hashCode => Object.hash(viewer, target);
}
