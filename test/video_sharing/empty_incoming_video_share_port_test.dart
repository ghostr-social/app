import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/sharing/empty_incoming_video_share_port.dart';

import '../support/sample_data.dart';

void main() {
  test('does nothing on platforms without incoming video sharing', () async {
    final port = EmptyIncomingVideoSharePort();
    final media = sampleMedia();

    expect(await port.events.toList(), isEmpty);
    await port.acknowledge(media);
    await port.release(media);
    await port.close();
  });
}
