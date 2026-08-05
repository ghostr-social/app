import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/data/playable_remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';

import '../support/fake_remote_video_source.dart';
import '../support/sample_data.dart';
import '../support/stub_video_gateways.dart';

Future<ProductionVideoDelivery> buildWithSources() async {
  final root = await Directory.systemTemp.createTemp('ghostr-rust-feed-');
  addTearDown(() => root.delete(recursive: true));
  final environment = ProductionVideoDeliveryEnvironment(
    source: FakeRemoteVideoSource([samplePost(id: 'rust-post')]),
    adapters: ProductionVideoDeliveryAdapters(
      supportDirectoryProvider: () async => root,
      gateway: startedVideoGateway(),
    ),
  );
  return buildProductionVideoDelivery(AppSettings.defaults(), environment);
}

Future<String> singlePostId(RemoteVideoSource source) async {
  final posts = await source.loadRemoteFeed();
  return posts.single.id.value;
}

void main() {
  test('production serves every feed path from Rust', () async {
    final delivery = await buildWithSources();

    expect(delivery.remoteSource, isA<PlayableRemoteVideoSource>());
    expect(delivery.searchSource, isA<PlayableRemoteVideoSource>());
    expect(await singlePostId(delivery.remoteSource), 'rust-post');
    expect(await singlePostId(delivery.searchSource), 'rust-post');
    expect(await singlePostId(delivery.discoverySource), 'rust-post');
  });
}
