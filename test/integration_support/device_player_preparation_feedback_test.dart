import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/device_player_preparation_feedback.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  test('device preparation tokens are unique across testbeds', () {
    final authority = testPlaybackAuthority();
    final first = DevicePlayerPreparationFeedback().prepare(authority);
    final second = DevicePlayerPreparationFeedback().prepare(authority);

    expect(first.nativeToken, isNotNull);
    expect(second.nativeToken, isNotNull);
    expect(first.nativeToken!.value, isNot(second.nativeToken!.value));
  });
}
