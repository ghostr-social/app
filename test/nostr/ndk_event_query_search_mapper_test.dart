import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_mapper.dart';

void main() {
  test('a query search term becomes the NIP-50 search field on the filter',
      () {
    const mapper = NdkNostrEventMapper();

    final searching = mapper.toFilter(
      NostrEventQuery(kinds: const [21], search: 'ghost dance'),
    );
    final browsing = mapper.toFilter(NostrEventQuery(kinds: const [21]));

    expect(searching.search, 'ghost dance');
    expect(browsing.search, isNull);
  });
}
