import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/pending_first_session_repository.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/pending_account_setup.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_account_provisioning_repository.dart';
import '../support/fake_session_repository.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('importing an existing key clears unfinished generated setup', () async {
    final provisioning = FakeAccountProvisioningRepository()
      ..pending = PendingAccountSetup(
        account: accountCreationAccount(),
        metadata: accountCreationMetadata(),
      );
    final inner = FakeSessionRepository();
    final repository = PendingFirstSessionRepository(inner, provisioning);

    final session = await repository.signIn(AuthSecret.parse(testNsec));

    expect(session, same(inner.storedSession));
    expect(provisioning.pending, isNull);
    expect(provisioning.discardCount, 1);
  });
}
