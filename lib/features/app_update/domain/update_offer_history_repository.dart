import 'package:ghostr/features/app_update/domain/android_version_code.dart';

abstract interface class UpdateOfferHistoryRepository {
  Future<AndroidVersionCode?> readLastDeclinedVersion();

  Future<void> recordDeclinedVersion(AndroidVersionCode version);
}
