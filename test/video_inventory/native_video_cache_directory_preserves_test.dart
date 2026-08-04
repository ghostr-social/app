import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/native_video_cache_directory.dart';

void main() {
  test('keeps already downloaded video bytes across a restart', () async {
    final root = await Directory.systemTemp.createTemp('ghostr-native-');
    addTearDown(() => root.delete(recursive: true));
    final cache = Directory('${root.path}/native_video_inventory');
    await Directory('${cache.path}/progressive').create(recursive: true);
    final stored = File('${cache.path}/progressive/post.part')
      ..writeAsStringSync('bytes');
    final stats = File('${cache.path}/host_stats.json')
      ..writeAsStringSync('{}');

    await NativeVideoCacheDirectory(cache).initialize();

    expect(stored.existsSync(), isTrue);
    expect(stats.existsSync(), isTrue);
  });
}
