import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';
import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/session/domain/user_session.dart';

import '../support/fakes.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('offers retry when secure session restoration fails', (
    tester,
  ) async {
    final dependencies = buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
      overrides: FakeDependencyOverrides(
        sessionRepository: _RestoreFailureRepository(),
      ),
    );

    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    expect(find.text('Secure session unavailable.'), findsOneWidget);
    expect(find.text('Retry'), findsOneWidget);
  });
}

class _RestoreFailureRepository implements SessionRepository {
  @override
  Future<UserSession?> restore() {
    throw const AppFailure('Secure session unavailable.');
  }

  @override
  Future<UserSession> signIn(AuthSecret secret) => throw UnimplementedError();

  @override
  Future<void> signOut() => throw UnimplementedError();

  @override
  Future<void> resetStoredSession() => throw UnimplementedError();
}
