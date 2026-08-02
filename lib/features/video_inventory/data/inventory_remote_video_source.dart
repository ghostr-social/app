import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_inventory/domain/video_inventory_port.dart';

class InventoryRemoteVideoSource implements RemoteVideoSource {
  const InventoryRemoteVideoSource({
    required RemoteVideoSource source,
    required VideoInventoryPort inventory,
  })  : _source = source,
        _inventory = inventory;

  final RemoteVideoSource _source;
  final VideoInventoryPort _inventory;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
  }) async {
    final posts = await _source.loadRemoteFeed(
      creatorIds: creatorIds,
      searchQuery: searchQuery,
    );
    _inventory.prepare(posts.map((post) => post.media).toList());
    return posts;
  }
}
