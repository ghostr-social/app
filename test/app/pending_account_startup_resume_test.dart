import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/domain/pending_account_setup.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_account_provisioning_repository.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('startup resumes unfinished account instead of opening Home', (
    tester,
  ) async {
    final provisioning = FakeAccountProvisioningRepository()
      ..pending = PendingAccountSetup(
        account: accountCreationAccount(),
        metadata: accountCreationMetadata(),
      );
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
      accountProvisioningRepository: provisioning,
    );

    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    expect(find.text('Back up your private key'), findsOneWidget);
    expect(find.text('Home'), findsNothing);
  });
}
