import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';
import 'package:ghostr/platform/media/ffi_hls_playback_gateway.dart';
import 'package:ghostr/src/rust/video/ffi_models.dart';

void main() {
  test('maps source mirrors to an owned loopback playback lease', () async {
    List<String>? capturedSources;
    final released = <String>[];
    final gateway = FfiHlsPlaybackGateway(
      acquireSession: ({required sourceUrls}) async {
        capturedSources = sourceUrls;
        return FfiHlsPlaybackSession(
          sessionId: 'a' * 64,
          playbackUrl: 'http://127.0.0.1:4242/hls/${'a' * 64}/index.m3u8',
        );
      },
      releaseSession: ({required sessionId}) async {
        released.add(sessionId);
        return true;
      },
    );
    final media = VideoMediaSource.remote(
      'https://media.example/master.m3u8',
      fallbackUrls: const ['https://mirror.example/master.m3u8'],
      delivery: VideoMediaDelivery.hls,
    );

    final lease = await gateway.acquire(HlsPlaybackRequest.fromMedia(media));

    expect(capturedSources, [
      'https://media.example/master.m3u8',
      'https://mirror.example/master.m3u8',
    ]);
    expect(lease.media.playbackUri.host, '127.0.0.1');
    lease.release();
    lease.release();
    await Future<void>.delayed(Duration.zero);
    expect(released, ['a' * 64]);
  });

  test('releases a native session whose playback URL is invalid', () async {
    final released = <String>[];
    final gateway = FfiHlsPlaybackGateway(
      acquireSession: ({required sourceUrls}) async =>
          const FfiHlsPlaybackSession(
        sessionId: 'native-session',
        playbackUrl: 'https://media.example/unsafe.m3u8',
      ),
      releaseSession: ({required sessionId}) async {
        released.add(sessionId);
        return true;
      },
    );
    final request = HlsPlaybackRequest.fromMedia(VideoMediaSource.remote(
      'https://media.example/master.m3u8',
      delivery: VideoMediaDelivery.hls,
    ));

    await expectLater(gateway.acquire(request), throwsFormatException);
    expect(released, ['native-session']);
  });

  test('contains rejected and failed native release attempts', () async {
    Future<bool> rejected({required String sessionId}) async => false;
    Future<bool> failed({required String sessionId}) async {
      throw StateError('native release failed');
    }

    for (final release in <RustHlsSessionReleaser>[rejected, failed]) {
      final gateway = FfiHlsPlaybackGateway(
        acquireSession: ({required sourceUrls}) async => FfiHlsPlaybackSession(
          sessionId: 'b' * 64,
          playbackUrl: 'http://127.0.0.1:4242/hls/${'b' * 64}/index.m3u8',
        ),
        releaseSession: release,
      );
      final request = HlsPlaybackRequest.fromMedia(VideoMediaSource.remote(
        'https://media.example/master.m3u8',
        delivery: VideoMediaDelivery.hls,
      ));
      final lease = await gateway.acquire(request);

      lease.release();
      await Future<void>.delayed(Duration.zero);
    }
  });
}
