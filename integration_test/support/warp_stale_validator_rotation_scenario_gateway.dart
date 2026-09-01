part of 'warp_stale_validator_rotation_scenario.dart';

extension _WarpValidatorRotationGateway on _WarpValidatorRotationDriver {
  Future<Uint8List> _readReplacementBytes() async {
    final state = graph.cubit.state as FeedLoaded;
    final post = state.posts.singleWhere(
      (item) => item.id.value == scenario.events.first.id,
    );
    final media = await const FfiProgressivePlaybackGateway().resolve(
      post.media,
    );
    final client = HttpClient();
    try {
      final request = await client.getUrl(media.playbackUri);
      final response = await request.close().timeout(_gatewayTimeout);
      expect(response.statusCode, HttpStatus.ok);
      final builder = await response
          .timeout(_gatewayTimeout)
          .fold<BytesBuilder>(
            BytesBuilder(copy: false),
            (result, chunk) => result..add(chunk),
          );
      return builder.takeBytes();
    } finally {
      client.close(force: true);
    }
  }
}

const _gatewayTimeout = Duration(seconds: 30);
