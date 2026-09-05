import 'package:flutter_test/flutter_test.dart';
import '../../integration_test/support/warp_request_occupancy.dart';

void main() {
  test('broker counters distinguish total and per-origin occupancy', () {
    final value = WarpRequestOccupancy.fromJson({
      'total': 2,
      'authorities': {'https://one.invalid': 1, 'https://two.invalid': 1},
      'invalid': 0,
    });
    expect(value.total, 2);
    expect(value.maximumPerOrigin, 1);
    expect(value.withinCoreBounds, isTrue);
  });

  test('same-origin overlap violates CORE even below the global ceiling', () {
    final value = WarpRequestOccupancy.fromJson({
      'total': 2,
      'authorities': {'https://one.invalid': 2},
      'invalid': 0,
    });
    expect(value.withinCoreBounds, isFalse);
  });

  test(
    'inconsistent or absent native occupancy cannot certify request limits',
    () {
      expect(() => WarpRequestOccupancy.fromJson({}), throwsFormatException);
      expect(
        () => WarpRequestOccupancy.fromJson({
          'total': 1,
          'authorities': {'https://one.invalid': 2},
          'invalid': 0,
        }),
        throwsFormatException,
      );
    },
  );
}
