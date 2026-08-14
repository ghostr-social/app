import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

void main() {
  test('tag filters reject blank values', () {
    expect(
      () => NostrTagFilter(name: 'a', values: const ['   ']),
      throwsFormatException,
    );
  });
}
