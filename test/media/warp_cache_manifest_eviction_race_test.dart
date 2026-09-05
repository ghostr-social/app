import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_cache_pressure_storage.dart';

void main() {
  test('a listed manifest can disappear during cache eviction', () async {
    final root = await Directory.systemTemp.createTemp(
      'warp-evicted-manifest-',
    );
    addTearDown(() => root.delete(recursive: true));
    final file = File('${root.path}/post.ranges.json');
    await file.writeAsString('{"intervals":[{"start":0,"end":32}]}');
    final listed = await root.list().single as File;
    await file.delete();

    expect(await readWarpCacheManifest(listed), isNull);
  });

  test('invalid retained metadata remains an observable failure', () async {
    final root = await Directory.systemTemp.createTemp(
      'warp-invalid-manifest-',
    );
    addTearDown(() => root.delete(recursive: true));
    final file = File('${root.path}/post.ranges.json');
    await file.writeAsString('invalid JSON');

    await expectLater(readWarpCacheManifest(file), throwsFormatException);
  });
}
