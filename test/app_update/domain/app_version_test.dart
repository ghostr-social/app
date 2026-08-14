import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/app_version.dart';

void main() {
  test('validates and orders semantic app versions', () {
    final versions = [
      '1.0.0',
      '0.1.0',
      '0.0.10',
      '0.0.9',
    ].map(AppVersion.parse).toList()..sort();

    expect(versions.map((version) => version.value), [
      '0.0.9',
      '0.0.10',
      '0.1.0',
      '1.0.0',
    ]);
    expect(AppVersion.tryParse('1.2'), isNull);
    expect(AppVersion.tryParse('v1.2.3'), isNull);
    expect(() => AppVersion.parse('01.2.3'), throwsFormatException);
    expect(AppVersion.tryParse('${'9' * 100}.0.0'), isNull);
    final version = AppVersion.parse('1.2.3');
    expect(version, AppVersion.parse('1.2.3'));
    expect(version.hashCode, '1.2.3'.hashCode);
    expect(version.toString(), '1.2.3');
  });
}
