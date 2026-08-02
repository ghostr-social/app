import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';

import '../support/settings_screen_harness.dart';

void main() {
  testWidgets('shows a retryable settings load error', (tester) async {
    final repository = _FailingSettingsRepository();
    await tester.pumpWidget(settingsScreenHarness(repository));
    await tester.pumpAndSettle();

    expect(find.text('Settings unavailable'), findsOneWidget);
    expect(find.text('Settings storage failed.'), findsOneWidget);
    await tester.tap(find.text('Retry'));
    await tester.pumpAndSettle();
    expect(repository.loadCount, 2);
  });
}

class _FailingSettingsRepository implements AppSettingsRepository {
  int loadCount = 0;

  @override
  Future<AppSettings> load() async {
    loadCount += 1;
    throw const AppFailure('Settings storage failed.');
  }

  @override
  Future<void> save(AppSettings settings) async {}
}
