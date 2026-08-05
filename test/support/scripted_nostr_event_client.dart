import 'dart:async';

import 'package:ghostr/core/nostr/nostr_event_record.dart';

import 'fake_nostr_event_client.dart';
import 'nostr_test_values.dart';

typedef NostrQueryScript = FutureOr<List<NostrEventRecord>> Function(
  NostrEventQuery query,
);

final class ScriptedNostrEventClient extends FakeNostrEventClient {
  ScriptedNostrEventClient(this._script)
      : super(publicKeyHex: testViewerPublicKey);

  final NostrQueryScript _script;

  @override
  Future<List<NostrEventRecord>> query(NostrEventQuery query) async {
    requestCount += 1;
    queries.add(query);
    return _script(query);
  }
}
