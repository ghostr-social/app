import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('does not expose mutable validated query collections', () {
    final authors = <NostrPublicKeyHex>[
      NostrPublicKeyHex.parse(testCreatorPublicKey),
    ];
    final scope = NostrEventQueryScope(authors: authors);
    final query = NostrEventQuery(kinds: [22], scope: scope);
    authors.clear();

    expect(scope.authors, hasLength(1));
    expect(() => scope.authors.clear(), throwsUnsupportedError);
    expect(() => query.kinds.clear(), throwsUnsupportedError);
  });
}
