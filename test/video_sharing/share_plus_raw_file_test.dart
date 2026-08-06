import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_sharing/domain/shareable_video_file.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';
import 'package:ghostr/platform/sharing/share_plus_video_file_port.dart';
import 'package:share_plus/share_plus.dart';

void main() {
  test('sends one local mp4 and no link to the platform share sheet', () async {
    late ShareParams captured;
    final port = SharePlusVideoFilePort(
      platformShare: (parameters) async {
        captured = parameters;
        return const ShareResult('whatsapp', ShareResultStatus.success);
      },
    );
    const origin = VideoShareOrigin(left: 1, top: 2, width: 3, height: 4);

    await port.share(
      ShareableVideoFile.parse('/tmp/downloaded.mp4'),
      origin: origin,
    );

    expect(captured.files?.single.path, '/tmp/downloaded.mp4');
    expect(captured.files?.single.mimeType, 'video/mp4');
    expect(captured.uri, isNull);
    expect(captured.text, isNull);
    expect(captured.sharePositionOrigin?.width, 3);
  });
}
