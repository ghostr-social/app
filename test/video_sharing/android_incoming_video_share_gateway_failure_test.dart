import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';

import '../support/fake_incoming_video_share_gateway.dart';

void main() {
  test('turns an Android gateway exception into a safe failure', () async {
    final gateway = FakeIncomingVideoShareGateway(
      takeFailure: StateError('native URI permission details'),
    );
    addTearDown(gateway.close);
    final port = AndroidIncomingVideoSharePort(gateway);

    final event = await port.events.first;

    final failure = event as IncomingVideoShareFailure;
    expect(failure.message, 'Could not open the shared video.');
    expect(failure.message, isNot(contains('permission')));
  });
}
