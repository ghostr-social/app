part of 'feed_preparation_reducer.dart';

PreparedProgressivePlayback? _matching(
  PlaybackPreparationAsset? asset,
  VideoMediaSource media,
) {
  return asset?.matches(media) == true ? asset!.bind(media) : null;
}

List<PreparedProgressivePlayback> _startableWindow(
  List<PlaybackPreparationAsset> assets,
  List<VideoMediaSource> media,
) {
  return media
      .map((source) => _startableAsset(assets, source))
      .whereType<PreparedProgressivePlayback>()
      .toList(growable: false);
}

PreparedProgressivePlayback? _startableAsset(
  List<PlaybackPreparationAsset> assets,
  VideoMediaSource media,
) {
  final asset = assets.where((value) => value.matches(media)).firstOrNull;
  if (asset?.readiness != PlaybackPreparationReadiness.structuralStartable) {
    return null;
  }
  return asset!.bind(media);
}

PreparedProgressivePlayback? _matchingAsset(
  List<PlaybackPreparationAsset> assets,
  VideoMediaSource media,
) {
  return assets.where((asset) => asset.matches(media)).firstOrNull?.bind(media);
}

PreparedProgressivePlayback? _matchingPrepared(
  List<PreparedProgressivePlayback> assets,
  VideoMediaSource media,
) {
  return assets.where((asset) => asset.matches(media)).firstOrNull;
}

List<PreparedProgressivePlayback> _retained(
  List<PreparedProgressivePlayback> assets,
  List<VideoMediaSource> media,
) {
  return media
      .map((source) => _matchingPrepared(assets, source))
      .whereType<PreparedProgressivePlayback>()
      .toList(growable: false);
}
