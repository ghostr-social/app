import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_nostr_account_generator.dart';

void main() {
  final account = accountCreationAccount();
  final metadata = accountCreationMetadata();
  final calls = <String>[];
  final sessions = RecordingSessionRepository(account.identity, calls: calls);
  final profiles = RecordingProfileRepository(calls: calls);

  blocTest<AccountCreationCubit, AccountCreationState>(
    'signs in before publishing and completes with the saved profile',
    build: () => AccountCreationCubit(
      FakeNostrAccountGenerator(account),
      sessions,
      profiles,
    ),
    act: (cubit) async {
      await cubit.begin(metadata);
      await cubit.complete();
    },
    expect: () => [
      isA<AccountCreationStaging>(),
      isA<AccountCreationAwaitingBackup>(),
      isA<AccountCreationProvisioning>(),
      isA<AccountCreationCompleted>().having(
        (state) => state.session.profile,
        'profile',
        same(profiles.savedProfile),
      ),
    ],
    verify: (cubit) {
      expect(calls, ['signIn', 'saveProfile']);
      expect(sessions.signInSecrets, [same(account.secret)]);
      expect(profiles.savedIdentity, same(account.identity));
      expect(profiles.savedMetadata, same(metadata));
    },
  );
}
