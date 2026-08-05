import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('generated Dart API excludes Rust start synchronization', () {
    final internalApi = File('lib/src/rust/api/runtime_registry.dart');

    expect(internalApi.existsSync(), isFalse);
  });
}
