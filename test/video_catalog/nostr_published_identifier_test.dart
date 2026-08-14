import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';

void main() {
  test('preserves exact non-blank identifiers used in coordinates', () {
    expect(NostrEventIdentifier.published(' clip ').value, ' clip ');
    expect(() => NostrEventIdentifier.published('   '), throwsFormatException);
  });
}
