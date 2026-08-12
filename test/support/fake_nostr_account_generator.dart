import 'package:ghostr/features/session/domain/generated_nostr_account.dart';
import 'package:ghostr/features/session/domain/nostr_account_generator.dart';

class FakeNostrAccountGenerator implements NostrAccountGenerator {
  FakeNostrAccountGenerator(this.account);

  final GeneratedNostrAccount account;
  int generationCount = 0;

  @override
  GeneratedNostrAccount generate() {
    generationCount += 1;
    return account;
  }
}
