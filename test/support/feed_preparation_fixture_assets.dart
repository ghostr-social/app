part of 'feed_preparation_fixture.dart';

VideoPost _preparationPost(int index) {
  final id = 'p$index';
  final remote = VideoMediaSource.remote('https://media.test/$id.mp4');
  return samplePost(
    id: id,
    caption: 'Caption $id',
  ).withMedia(VideoMediaSource.withCacheScope(remote, id));
}

PlaybackPreparationAsset _preparationAsset(List<VideoPost> posts, String id) {
  final capability = switch (id) {
    'p0' => _capA,
    'p1' => _capB,
    _ => _capC,
  };
  return PlaybackPreparationAsset(
    authority: PlaybackAssetAuthority(
      deliveryId: PlaybackDeliveryId.parse(id),
      representationId: VideoRepresentationId.forMedia(
        posts.singleWhere((post) => post.media.cacheScope?.value == id).media,
      ),
      assetId: PlaybackAssetId.parse(capability),
    ),
    media: ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:4040/video.mp4?id=$id&cap=$capability',
    ),
    readiness: PlaybackPreparationReadiness.structuralStartable,
  );
}

const _capA = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const _capB = 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
const _capC = 'ccccccccccccccccccccccccccccccccccccccccccc';
