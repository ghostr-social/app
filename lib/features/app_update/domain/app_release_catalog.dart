import 'package:ghostr/features/app_update/domain/stable_release.dart';

abstract interface class AppReleaseCatalog {
  Future<StableRelease> fetchStableRelease();
}
