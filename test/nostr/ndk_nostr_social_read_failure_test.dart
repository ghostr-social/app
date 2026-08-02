import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:mocktail/mocktail.dart';
import 'package:ndk/ndk.dart';

import '../support/ndk_mocks.dart';

void main() {
  test('translates social-list read failures into app-safe failures', () async {
    final ndk = MockNdk();
    final accounts = MockAccounts();
    final lists = MockLists();
    final follows = MockFollows();
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.lists).thenReturn(lists);
    when(() => ndk.follows).thenReturn(follows);
    when(accounts.getPublicKey).thenReturn('viewer');
    when(
      () => lists.getSingleNip51List(Nip51List.kMute),
    ).thenThrow(StateError('list offline'));
    when(
      () => follows.getContactList('viewer'),
    ).thenThrow(StateError('contacts offline'));
    final runtime = NdkNostrSocial(ndk: ndk, relays: const []);

    await expectLater(
      runtime.loadBlockedProfiles(),
      throwsA(isA<AppFailure>()),
    );
    await expectLater(
      runtime.loadFollowedProfiles(),
      throwsA(isA<AppFailure>()),
    );
  });
}
