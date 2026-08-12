import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('Android sources use the Ghostr package directory', () {
    const legacyName =
        'tok'
        'str';
    for (final sourceSet in ['main', 'test']) {
      final packageRoot = Directory('android/app/src/$sourceSet/kotlin/app');
      final ghostrRoot = Directory('${packageRoot.path}/ghostr');
      final legacyRoot = Directory('${packageRoot.path}/$legacyName');

      expect(legacyRoot.existsSync(), isFalse, reason: legacyRoot.path);
      expect(ghostrRoot.existsSync(), isTrue, reason: ghostrRoot.path);
      final sources = ghostrRoot
          .listSync(recursive: true)
          .whereType<File>()
          .where((file) => file.path.endsWith('.kt'));
      expect(sources, isNotEmpty, reason: ghostrRoot.path);
      for (final source in sources) {
        expect(source.readAsStringSync(), contains('package app.ghostr'));
      }
    }
  });
}
