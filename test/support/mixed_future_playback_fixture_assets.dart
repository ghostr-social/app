part of 'mixed_future_playback_fixture.dart';

List<VideoPost> _mixedPosts() => [
  for (var index = 0; index < 4; index++) _progressivePost('p$index'),
  for (var index = 0; index < 3; index++) _hlsPost('h$index'),
  for (var index = 4; index < 9; index++) _progressivePost('p$index'),
];

VideoPost _progressivePost(String id) => samplePost(id: id).withMedia(
  VideoMediaSource.withCacheScope(
    VideoMediaSource.remote('https://media.test/$id.mp4'),
    id,
  ),
);

VideoPost _hlsPost(String id) => samplePost(id: id).withMedia(
  VideoMediaSource.withCacheScope(
    VideoMediaSource.remote(
      'https://media.test/$id/master.m3u8',
      delivery: VideoMediaDelivery.hls,
    ),
    id,
  ),
);

bool _isHlsPost(VideoPost post) {
  return post.media.remoteDelivery == VideoMediaDelivery.hls;
}

PlaybackPreparationAsset _mixedPreparationAsset(
  List<VideoPost> posts,
  String id,
) {
  final origin = posts.singleWhere((post) => post.id.value == id).media;
  return PlaybackPreparationAsset(
    authority: PlaybackAssetAuthority(
      deliveryId: PlaybackDeliveryId.parse(id),
      representationId: VideoRepresentationId.forMedia(origin),
      assetId: PlaybackAssetId.parse(_mixedCapability),
    ),
    media: ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:4040/video.mp4?id=$id&cap=$_mixedCapability',
    ),
    readiness: PlaybackPreparationReadiness.structuralStartable,
  );
}

HlsPlaybackAuthority _mixedHlsAuthority(VideoPost post) {
  return HlsPlaybackAuthority(
    deliveryId: post.media.playbackDeliveryId!,
    representationId: VideoRepresentationId.forMedia(post.media),
    assetRevision: HlsPlaybackAssetRevision.parse(BigInt.one),
  );
}

const _mixedCapability = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
