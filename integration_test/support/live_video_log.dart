import 'dart:convert';

import 'package:flutter/foundation.dart';

final class LiveVideoLog {
  final watch = Stopwatch()..start();
  final records = <Map<String, Object?>>[];
  int dropped = 0;

  void add(String type, Map<String, Object?> fields) {
    final record = <String, Object?>{
      'type': type,
      'elapsedMs': watch.elapsedMilliseconds,
      ...fields,
    };
    if (records.length >= 6000) {
      records.removeAt(0);
      dropped++;
    }
    records.add(record);
    debugPrintSynchronously('WARP_LIVE ${jsonEncode(record)}');
  }

  Map<String, Object?> report() => {
    'records': records,
    'droppedRecords': dropped,
    'elapsedMs': watch.elapsedMilliseconds,
    'flutterMode': kProfileMode ? 'profile' : 'debug',
  };
}
