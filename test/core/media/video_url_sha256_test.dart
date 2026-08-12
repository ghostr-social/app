import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_url_sha256.dart';

const _digest =
    'f2ca1bb6c7e907d06dafe4687e579fce76b37e4e93b7605022da52e6ccc26fd2';

void main() {
  test('reads a blossom-style digest out of a media URL', () {
    expect(
      inferVideoSha256FromUrl('https://cdn.example/$_digest.mp4')?.value,
      _digest,
    );
    expect(
      inferVideoSha256FromUrl('https://cdn.example/media/$_digest')?.value,
      _digest,
    );
    expect(
      inferVideoSha256FromUrl(
        'https://cdn.example/${_digest.toUpperCase()}.MP4',
      )?.value,
      _digest,
    );
    expect(inferVideoSha256FromUrl('https://cdn.example/clip.mp4'), isNull);
    expect(inferVideoSha256FromUrl('https://cdn.example/'), isNull);
    expect(inferVideoSha256FromUrl('::not a url::'), isNull);
  });
}
