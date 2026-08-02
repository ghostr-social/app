import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/compose/presentation/compose_state.dart';

void main() {
  test('cannot transition an empty composer to publishing', () {
    expect(const ComposeState.idle().publishing, throwsStateError);
  });
}
