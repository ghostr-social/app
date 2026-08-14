import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

void main() {
  test('tag filters preserve exact nonblank values', () {
    final filter = NostrTagFilter(name: 'a', values: const [' coordinate ']);

    expect(filter.values, const [' coordinate ']);
  });
}
