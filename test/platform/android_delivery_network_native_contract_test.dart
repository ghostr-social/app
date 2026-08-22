import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('Android publishes authoritative delivery transport generations', () {
    final kotlin = Directory('android/app/src/main/kotlin/social/ghostr')
        .listSync(recursive: true)
        .whereType<File>()
        .map((file) => file.readAsStringSync())
        .join('\n');

    expect(kotlin, contains('social.ghostr/network/v1'));
    expect(kotlin, contains('registerDefaultNetworkCallback'));
    expect(kotlin, contains('TRANSPORT_WIFI'));
    expect(kotlin, contains('TRANSPORT_CELLULAR'));
    expect(kotlin, contains('TRANSPORT_ETHERNET'));
    expect(kotlin, contains('NET_CAPABILITY_VALIDATED'));
    expect(kotlin, contains('NET_CAPABILITY_NOT_RESTRICTED'));
    expect(kotlin, contains('RESTRICT_BACKGROUND_STATUS_ENABLED'));
    expect(kotlin, contains('AtomicLong'));
    expect(kotlin, contains('incrementAndGet'));
    expect(kotlin, contains('networkStatusChanged'));
    expect(kotlin, contains('generation'));
  });
}
