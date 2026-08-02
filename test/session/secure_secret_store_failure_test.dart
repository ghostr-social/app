import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/storage/secure_secret_store.dart';
import 'package:mocktail/mocktail.dart';

import '../support/mock_flutter_secure_storage.dart';

void main() {
  test('translates secure-storage read failures into an app-safe failure',
      () async {
    final storage = MockFlutterSecureStorage();
    when(() => storage.read(key: any(named: 'key')))
        .thenThrow(StateError('keystore unavailable'));

    final future = SecureSecretStore(storage).read();

    await expectLater(
      future,
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        'Secure storage is unavailable.',
      )),
    );
  });
}
