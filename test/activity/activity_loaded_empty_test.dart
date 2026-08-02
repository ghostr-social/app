import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/presentation/activity_cubit.dart';

void main() {
  test('rejects an empty loaded activity state', () {
    expect(() => ActivityLoaded(const []), throwsStateError);
  });
}
