import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/session/domain/user_session.dart';

import '../support/fakes.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('clears a broken stored key and returns to account access', (
    tester,
  ) async {
    final repository = _RestoreFailureRepository();
    final dependencies = buildFakeDependencies(
      sessionRepository: repository,
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
    );

    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Use another key'));
    await tester.pumpAndSettle();

    expect(repository.clearCount, 1);
    expect(find.text('Welcome to Ghostr'), findsOneWidget);
    expect(find.text('Create a Nostr account'), findsOneWidget);
    expect(find.text('Use an existing key'), findsOneWidget);
  });
}

class _RestoreFailureRepository implements SessionRepository {
  int clearCount = 0;

  @override
  Future<UserSession?> restore() {
    throw const AppFailure('Secure session unavailable.');
  }

  @override
  Future<UserSession> signIn(AuthSecret secret) => throw UnimplementedError();

  @override
  Future<void> resetStoredSession() async {
    clearCount += 1;
  }

  @override
  Future<void> signOut() => throw UnimplementedError();
}
