import 'package:ghostr/core/media/playback_delivery_id.dart';

final testCanonicalPlaybackDeliveryId = PlaybackDeliveryId.parse('delivery');
const testPlaybackDeliveryId =
    '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
const testHlsPlaybackUrl =
    'http://127.0.0.1:8080/hls/$testPlaybackDeliveryId/index.m3u8';
