import 'package:ghostr/features/app_update/domain/installed_app.dart';

abstract interface class InstalledAppPort {
  Future<InstalledApp> readInstalledApp();
}
