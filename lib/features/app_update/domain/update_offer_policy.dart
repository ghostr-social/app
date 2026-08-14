import 'package:ghostr/features/app_update/domain/android_version_code.dart';

final class UpdateOfferPolicy {
  const UpdateOfferPolicy();

  bool shouldOffer({
    required AndroidVersionCode release,
    AndroidVersionCode? lastDeclined,
  }) {
    return lastDeclined == null || release.compareTo(lastDeclined) > 0;
  }
}
