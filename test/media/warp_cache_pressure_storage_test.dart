import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_cache_pressure_storage.dart';

void main() {
  test(
    'cache coverage counts committed manifest intervals by delivery',
    () async {
      final support = await Directory.systemTemp.createTemp(
        'warp-cache-audit-',
      );
      addTearDown(() => support.delete(recursive: true));
      final store = Directory(
        '${support.path}/native_video_inventory/progressive',
      );
      await store.create(recursive: true);
      await _manifest(store, 'cold', const [(start: 0, end: 40)]);
      await _manifest(store, 'warm', const [
        (start: 10, end: 30),
        (start: 50, end: 90),
      ]);

      final coverage = await readWarpCacheCoverage(support);

      expect(coverage.totalBytes, 100);
      expect(coverage.bytesFor('cold'), 40);
      expect(coverage.bytesFor('warm'), 60);
      expect(coverage.bytesFor('missing'), 0);
    },
  );
}

Future<void> _manifest(
  Directory store,
  String delivery,
  List<({int start, int end})> intervals,
) {
  final json = jsonEncode({
    'version': 2,
    'total_len': 100,
    'intervals': [
      for (final range in intervals)
        {'start': range.start, 'end': range.end, 'sha256': '0' * 64},
    ],
  });
  return File('${store.path}/$delivery.ranges.json').writeAsString(json);
}
