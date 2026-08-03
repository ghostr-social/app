import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:mocktail/mocktail.dart';

import '../support/ndk_mocks.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('translates an NDK metadata failure at the adapter boundary', () async {
    final ndk = MockNdk();
    final metadatas = MockMetadatas();
    when(() => ndk.metadata).thenReturn(metadatas);
    when(() => metadatas.loadMetadatas([testCreatorPublicKey], null)).thenThrow(
      StateError('metadata socket failed'),
    );

    await expectLater(
      NdkNostrVideoEventQuery(ndk).loadMetadataBatch({
        NostrPublicKeyHex.parse(testCreatorPublicKey),
      }),
      throwsA(isA<AppFailure>()),
    );
  });
}
