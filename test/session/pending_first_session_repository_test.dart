import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/pending_first_session_repository.dart';
import 'package:ghostr/features/session/domain/pending_account_setup.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_session_repository.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'unfinished generated account takes priority over active session',
    () async {
      final provisioning =
          RecordingSessionRepository(accountCreationAccount().identity)
            ..pending = PendingAccountSetup(
              account: accountCreationAccount(),
              metadata: accountCreationMetadata(),
            );
      final repository = PendingFirstSessionRepository(
        FakeSessionRepository(storedSession: sampleSession()),
        provisioning,
      );

      expect(await repository.restore(), isNull);

      await provisioning.discard();
      expect(await repository.restore(), isNotNull);
    },
  );
}
