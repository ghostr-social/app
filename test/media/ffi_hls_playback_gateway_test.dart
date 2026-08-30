import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';
import 'package:ghostr/platform/media/ffi_hls_playback_gateway.dart';
import '../support/fake_hls_playback_gateway.dart';

void main() {
  test('maps prepared authority to an owned loopback lease', () async {
    final released = <String>[];
    final media = VideoMediaSource.withCacheScope(
      VideoMediaSource.remote(
        'https://media.example/master.m3u8',
        fallbackUrls: const ['https://mirror.example/master.m3u8'],
        delivery: VideoMediaDelivery.hls,
      ),
      'post-A',
    );
    final authority = HlsPlaybackAuthority(
      deliveryId: media.playbackDeliveryId!,
      representationId: VideoRepresentationId.forMedia(media),
      assetRevision: HlsPlaybackAssetRevision.parse(BigInt.from(7)),
    );
    final acquirer = RecordingRustHlsSessionAcquirer(
      sessionId: 'a' * 64,
      playbackUrl: 'http://127.0.0.1:4242/hls/${'a' * 64}/index.m3u8',
    );
    final gateway = FfiHlsPlaybackGateway(
      acquireSession: acquirer.call,
      releaseSession: ({required sessionId}) async {
        released.add(sessionId);
        return true;
      },
    );
    final lease = await gateway.acquire(
      HlsPlaybackRequest.fromMedia(media, expectedAuthority: authority),
    );
    expect(acquirer.sourceUrls, media.remoteUrls);
    expect(acquirer.deliveryId, authority.deliveryId.value);
    expect(acquirer.representationId, authority.representationId.value);
    expect(acquirer.assetRevision, authority.assetRevision.value);
    expect(lease.authority, authority);
    expect(lease.deliveryId, authority.deliveryId);
    expect(lease.media.playbackUri.host, '127.0.0.1');
    lease.release();
    lease.release();
    await Future<void>.delayed(Duration.zero);
    expect(released, ['a' * 64]);
  });

  test('releases a native session whose playback URL is invalid', () async {
    final released = <String>[];
    final acquirer = RecordingRustHlsSessionAcquirer(
      sessionId: 'native-session',
      playbackUrl: 'https://media.example/unsafe.m3u8',
    );
    final gateway = FfiHlsPlaybackGateway(
      acquireSession: acquirer.call,
      releaseSession: ({required sessionId}) async {
        released.add(sessionId);
        return true;
      },
    );
    final request = HlsPlaybackRequest.fromMedia(_legacyMedia());

    await expectLater(gateway.acquire(request), throwsFormatException);
    expect(released, ['native-session']);
  });

  test('contains rejected and failed native release attempts', () async {
    Future<bool> rejected({required String sessionId}) async => false;
    Future<bool> failed({required String sessionId}) async =>
        throw StateError('native release failed');
    for (final release in <RustHlsSessionReleaser>[rejected, failed]) {
      final acquirer = RecordingRustHlsSessionAcquirer(
        sessionId: 'b' * 64,
        playbackUrl: 'http://127.0.0.1:4242/hls/${'b' * 64}/index.m3u8',
      );
      final gateway = FfiHlsPlaybackGateway(
        acquireSession: acquirer.call,
        releaseSession: release,
      );
      final lease = await gateway.acquire(
        HlsPlaybackRequest.fromMedia(_legacyMedia()),
      );

      expect(lease.authority, isNull);
      lease.release();
      await Future<void>.delayed(Duration.zero);
    }
  });
}

VideoMediaSource _legacyMedia() => VideoMediaSource.remote(
  'https://media.example/master.m3u8',
  delivery: VideoMediaDelivery.hls,
);
