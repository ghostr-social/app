import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/app/production_app_update.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';

import '../support/fake_update_offer_history_repository.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test(
    'production environment composes Android update runtime when supported',
    () async {
      final environment = ProductionDependenciesEnvironment.production();
      final builder = environment.appUpdateBuilder;
      expect(builder, isNotNull);

      final runtime = builder!(
        _UnusedSettingsRepository(),
        FakeUpdateOfferHistoryRepository(),
      );
      expect(runtime, isA<AppUpdateRuntime>());
      await runtime.dispose();
    },
  );
}

final class _UnusedSettingsRepository implements AppSettingsRepository {
  @override
  Future<AppSettings> load() => throw UnimplementedError();

  @override
  Future<void> save(AppSettings settings) => throw UnimplementedError();
}
