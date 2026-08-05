import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/platform/nostr/rust_nostr_event_client.dart';
import 'package:ghostr/src/rust/api/event_types.dart';

import '../support/ndk_mocks.dart';
import '../support/recording_signed_event_broadcast_port.dart';

void main() {
  test('rejects more than twenty filters before calling Rust', () async {
    var calls = 0;
    final client = _client((_) => calls += 1);
    final queries = List<NostrEventQuery>.generate(
      21,
      (index) => NostrEventQuery(kinds: <int>[index + 1]),
    );

    await expectLater(
      client.queryBatch(queries),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          'Nostr filter batch exceeds the safe limit.',
        ),
      ),
    );
    expect(calls, 0);
  });

  test('returns an empty batch without calling Rust', () async {
    var calls = 0;
    final client = _client((_) => calls += 1);

    expect(await client.queryBatch(const <NostrEventQuery>[]), isEmpty);
    expect(calls, 0);
  });

  test('sends twenty mapped filters in one Rust batch', () async {
    final sent = <List<FfiNostrEventFilter>>[];
    final client = _client(sent.add);
    final queries = List<NostrEventQuery>.generate(
      20,
      (index) => NostrEventQuery(kinds: <int>[index + 1]),
    );

    expect(await client.queryBatch(queries), isEmpty);
    expect(sent.single, hasLength(20));
    expect(sent.single.last.kinds, <int>[20]);
  });
}

RustNostrEventClient _client(
  void Function(List<FfiNostrEventFilter>) onQuery,
) {
  return RustNostrEventClient(
    ndk: MockNdk(),
    broadcast: RecordingSignedEventBroadcastPort(),
    queries: RustNostrEventQueries(
      batch: ({required filters}) async {
        onQuery(filters);
        return const <FfiNostrEvent>[];
      },
    ),
  );
}
