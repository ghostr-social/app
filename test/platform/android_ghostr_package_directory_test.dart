import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('Android sources use the Ghostr package directory', () {
    const legacyName =
        'tok'
        'str';
    for (final sourceSet in ['main', 'test']) {
      final kotlinRoot = Directory('android/app/src/$sourceSet/kotlin');
      final ghostrRoot = Directory('${kotlinRoot.path}/social/ghostr');
      final oldApplicationIdRoot = Directory('${kotlinRoot.path}/app/ghostr');
      final legacyRoot = Directory('${kotlinRoot.path}/app/$legacyName');

      expect(legacyRoot.existsSync(), isFalse, reason: legacyRoot.path);
      expect(
        oldApplicationIdRoot.existsSync(),
        isFalse,
        reason: oldApplicationIdRoot.path,
      );
      expect(ghostrRoot.existsSync(), isTrue, reason: ghostrRoot.path);
      final sources = ghostrRoot
          .listSync(recursive: true)
          .whereType<File>()
          .where((file) => file.path.endsWith('.kt'));
      expect(sources, isNotEmpty, reason: ghostrRoot.path);
      for (final source in sources) {
        expect(source.readAsStringSync(), contains('package social.ghostr'));
      }
    }
  });
}
