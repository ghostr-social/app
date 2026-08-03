import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('skips a malformed metadata entry without losing valid entries',
      () async {
    final ndk = MockNdk();
    final metadatas = MockMetadatas();
    when(() => ndk.metadata).thenReturn(metadatas);
    when(() => metadatas.loadMetadatas(any(), null)).thenAnswer(
      (_) async => [
        Metadata(pubKey: 'malformed', displayName: 'Invalid'),
        Metadata(pubKey: testViewerPublicKey, displayName: 'Unexpected'),
        Metadata(pubKey: testCreatorPublicKey, displayName: 'Creator'),
      ],
    );
    final publicKey = NostrPublicKeyHex.parse(testCreatorPublicKey);

    final result = await NdkNostrVideoEventQuery(ndk).loadMetadataBatch({
      publicKey,
    });

    expect(result.keys, [publicKey]);
    expect(result[publicKey]?.getName(), 'Creator');
  });
}
