import 'dart:convert';
import 'dart:io';

final class WarpCacheCoverage {
  const WarpCacheCoverage(this.byDelivery);

  final Map<String, int> byDelivery;

  int get totalBytes => byDelivery.values.fold(0, _sum);

  int bytesFor(String deliveryId) => byDelivery[deliveryId] ?? 0;
}

Future<WarpCacheCoverage> readWarpCacheCoverage(Directory support) async {
  final root = Directory('${support.path}/native_video_inventory/progressive');
  if (!await root.exists()) return const WarpCacheCoverage({});
  final result = <String, int>{};
  await for (final entity in root.list()) {
    if (entity is! File || !entity.path.endsWith('.ranges.json')) continue;
    final entry = await _readManifest(entity);
    result[entry.deliveryId] = entry.coveredBytes;
  }
  return WarpCacheCoverage(Map.unmodifiable(result));
}

Future<({String deliveryId, int coveredBytes})> _readManifest(File file) async {
  final name = file.uri.pathSegments.last;
  const suffix = '.ranges.json';
  final json = jsonDecode(await file.readAsString()) as Map<String, Object?>;
  final intervals = json['intervals']! as List<Object?>;
  return (
    deliveryId: name.substring(0, name.length - suffix.length),
    coveredBytes: intervals.fold(0, _covered),
  );
}

int _covered(int total, Object? raw) {
  final interval = raw! as Map<String, Object?>;
  final start = interval['start']! as int;
  final end = interval['end']! as int;
  return total + end - start;
}

int _sum(int total, int value) => total + value;
