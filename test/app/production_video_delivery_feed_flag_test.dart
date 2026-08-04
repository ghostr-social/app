import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/feed_pipeline_flag.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';

import '../support/fake_remote_video_source.dart';
import '../support/fake_video_file_downloader.dart';
import '../support/sample_data.dart';
import '../support/stub_video_gateways.dart';

Future<ProductionVideoDelivery> buildWithFlag(FeedPipelineFlag flag) async {
  final root = await Directory.systemTemp.createTemp('ghostr-feed-flag-');
  addTearDown(() => root.delete(recursive: true));
  final environment = ProductionVideoDeliveryEnvironment(
    canonicalSource: FakeRemoteVideoSource([samplePost(id: 'ndk-post')]),
    supportDirectoryProvider: () async => root,
    downloader: FakeVideoFileDownloader({}),
    gateway: startedVideoGateway(),
    feedFlag: flag,
    rustFeedSourceBuilder: () =>
        FakeRemoteVideoSource([samplePost(id: 'rust-post')]),
  );
  return buildProductionVideoDelivery(AppSettings.defaults(), environment);
}

Future<String> singlePostId(RemoteVideoSource source) async {
  final posts = await source.loadRemoteFeed();
  return posts.single.id.value;
}

void main() {
  test('the default flag keeps every feed path on the ndk source', () async {
    final delivery = await buildWithFlag(const FeedPipelineFlag());

    expect(await singlePostId(delivery.remoteSource), 'ndk-post');
    expect(await singlePostId(delivery.discoverySource), 'ndk-post');
  });

  test('the rust flag serves every feed path from the rust source', () async {
    final delivery =
        await buildWithFlag(const FeedPipelineFlag(FeedPipelineMode.rust));

    expect(await singlePostId(delivery.remoteSource), 'rust-post');
    expect(await singlePostId(delivery.searchSource), 'rust-post');
    expect(await singlePostId(delivery.discoverySource), 'rust-post');
  });
}
