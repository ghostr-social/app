import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/update_offer_policy.dart';

void main() {
  test('offers only a release above the highest declined version', () {
    const policy = UpdateOfferPolicy();
    final declined = AndroidVersionCode(23);

    expect(
      policy.shouldOffer(
        release: AndroidVersionCode(23),
        lastDeclined: declined,
      ),
      isFalse,
    );
    expect(
      policy.shouldOffer(
        release: AndroidVersionCode(22),
        lastDeclined: declined,
      ),
      isFalse,
    );
    expect(
      policy.shouldOffer(
        release: AndroidVersionCode(24),
        lastDeclined: declined,
      ),
      isTrue,
    );
    expect(policy.shouldOffer(release: AndroidVersionCode(23)), isTrue);
  });
}
