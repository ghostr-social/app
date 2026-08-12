import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('native check promotes Rust warnings to errors', () {
    final makefile = File('Makefile').readAsStringSync();

    expect(
      makefile,
      contains(
        'cargo clippy --workspace --all-targets --all-features -- -D warnings',
      ),
    );
  });
}
