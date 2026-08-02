import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_inventory/domain/video_delivery_plan.dart';

void main() {
  test('does not expose mutable configured relays', () {
    final plan = VideoDeliveryPlan.fromSettings(AppSettings.defaults());

    expect(() => plan.relayUrls.clear(), throwsUnsupportedError);
  });
}
