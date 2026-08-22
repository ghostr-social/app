import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_metadata.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';

void main() {
  test('matches Rust representation identity vectors', () {
    expect(_advertised().value, _advertisedVector);
    expect(_unverified().value, _unverifiedVector);
    expect(_unicodeOrdered().value, _unicodeOrderVector);
  });
}

VideoRepresentationId _advertised() {
  final source = VideoMediaSource.remote('https://ignored.test/video.mp4');
  final verified = VideoMediaSource.withExpectedSha256(source, 'a' * 64);
  return VideoRepresentationId.forMedia(verified);
}

VideoRepresentationId _unverified() {
  final source = VideoMediaSource.remote(
    'https://b.test/video.mp4',
    fallbackUrls: const [
      'https://a.test/video.mp4',
      'https://b.test/video.mp4',
    ],
    metadata: const VideoMediaMetadata(sizeBytes: 123456, durationMs: 9876),
  );
  return VideoRepresentationId.forMedia(source);
}

VideoRepresentationId _unicodeOrdered() {
  return VideoRepresentationId.forMedia(
    VideoMediaSource.remote(
      'https://media.test/\u{10000}.mp4',
      fallbackUrls: const ['https://media.test/\u{e000}.mp4'],
    ),
  );
}

const _advertisedVector =
    '40b13ee390bb98651d749f074546e825c163cc0886d7a5ce51210cbbf6e761da';
const _unverifiedVector =
    'ede223d32401527e82b2e523f2a5ede1837019d2b46551995acefb2e2b0b70ea';
const _unicodeOrderVector =
    '2ec09b10cf5150d379803130724d865cbb743b4c32048a6b3f25431cd56c5d51';
