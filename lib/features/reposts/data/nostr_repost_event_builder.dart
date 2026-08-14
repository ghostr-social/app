import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

const _genericRepostKinds = <int>{21, 22, 1063, 34235, 34236};

int repostKindFor(NostrEventReference reference) {
  if (reference.kind.value == 1) return 6;
  if (_genericRepostKinds.contains(reference.kind.value)) return 16;
  throw const AppFailure('This Nostr event kind cannot be reposted.');
}

NostrUnsignedEvent buildRepostEvent(
  NostrEventReference reference,
  String? relayHint,
) {
  final kind = repostKindFor(reference);
  final hint = _validRelayHint(relayHint);
  if (kind == 6 && hint == null) {
    throw const AppFailure('A relay hint is required to repost this video.');
  }
  return NostrUnsignedEvent(
    kind: kind,
    tags: _targetTags(reference, hint),
    content: _content(reference),
  );
}

NostrUnsignedEvent buildRepostDeletion(
  Set<NostrEventId> repostIds,
  int repostKind,
) {
  return NostrUnsignedEvent(
    kind: 5,
    tags: <List<String>>[
      for (final id in repostIds) <String>['e', id.value],
      <String>['k', '$repostKind'],
    ],
    content: 'Removed repost',
  );
}

List<List<String>> _targetTags(NostrEventReference reference, String? hint) {
  return <List<String>>[
    <String>['e', reference.eventId.value, if (hint != null) hint],
    <String>['p', reference.authorPublicKeyHex.value],
    <String>['k', '${reference.kind.value}'],
    if (reference.coordinateIdentifier case final identifier?)
      <String>[
        'a',
        '${reference.kind}:${reference.authorPublicKeyHex}:$identifier',
      ],
  ];
}

String _content(NostrEventReference reference) {
  if (reference.isProtected) return '';
  final source = reference.signedEvent;
  if (source == null) {
    throw const AppFailure('The signed original event is unavailable.');
  }
  return source.value;
}

String? _validRelayHint(String? raw) {
  if (raw == null) return null;
  final uri = Uri.tryParse(raw.trim());
  if (!_usableRelayUri(uri)) return null;
  return uri.toString();
}

bool _usableRelayUri(Uri? uri) {
  if (uri == null || !uri.hasAuthority) return false;
  return _isRelayScheme(uri.scheme);
}

bool _isRelayScheme(String scheme) => scheme == 'ws' || scheme == 'wss';
