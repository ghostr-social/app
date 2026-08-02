import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/blossom_server_url.dart';

void main() {
  test('compares canonical Blossom servers by URL value', () {
    final first = BlossomServerUrl.parse('https://media.example/');
    final same = BlossomServerUrl.parse('https://media.example');
    final other = BlossomServerUrl.parse('https://other.example');

    expect(first, same);
    expect(first.hashCode, same.hashCode);
    expect(first, isNot(other));
  });
}
