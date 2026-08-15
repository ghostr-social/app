import 'package:ghostr/app/production_app_update.dart';
import 'package:ghostr/app/production_nostr_services.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/features/settings/data/local_app_settings_repository.dart';
import 'package:sembast/sembast.dart';
import 'package:shared_preferences/shared_preferences.dart';

final class ProductionDependencyInputs {
  const ProductionDependencyInputs({
    required this.preferences,
    required this.settingsRepository,
    required this.watchHistoryDatabase,
    required this.nostr,
    required this.delivery,
    required this.appUpdateRuntime,
  });

  final SharedPreferences preferences;
  final LocalAppSettingsRepository settingsRepository;
  final Database watchHistoryDatabase;
  final ProductionNostrServices nostr;
  final ProductionVideoDelivery delivery;
  final AppUpdateRuntime? appUpdateRuntime;
}
