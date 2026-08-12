import 'package:ghostr/features/session/domain/generated_nostr_account.dart';

abstract interface class NostrAccountGenerator {
  GeneratedNostrAccount generate();
}
