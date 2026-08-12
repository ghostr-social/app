import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('native test gate selects the debug web test owning package', () {
    final makefile = File('Makefile').readAsStringSync();

    expect(
      makefile,
      contains(
        'cargo test -p ghostr-gateway --no-default-features '
        '--test debug_web_exclusion_test',
      ),
    );
  });
}
