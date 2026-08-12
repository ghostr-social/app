import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/domain/generated_nostr_account.dart';
import 'package:ghostr/features/session/domain/nostr_account_generator.dart';
import 'package:ghostr/features/session/presentation/account_creation_cubit.dart';

import '../support/account_creation_fakes.dart';

void main() {
  test('translates an unexpected key generation failure safely', () async {
    final account = accountCreationAccount();
    final cubit = AccountCreationCubit(
      _UnexpectedGenerator(),
      RecordingSessionRepository(account.identity),
      RecordingProfileRepository(),
    );
    addTearDown(cubit.close);

    await expectLater(cubit.begin(accountCreationMetadata()), completes);
    expect(
      cubit.state,
      isA<AccountCreationIdle>().having(
        (state) => state.message,
        'message',
        'Could not generate a secure Nostr key.',
      ),
    );
  });
}

final class _UnexpectedGenerator implements NostrAccountGenerator {
  @override
  GeneratedNostrAccount generate() => throw StateError('random source failed');
}
