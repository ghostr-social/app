import 'package:ghostr/core/nostr/nostr_event_record.dart';

bool isNostrLikeReaction(NostrEventRecord event) {
  return event.kind == 7 && (event.content.isEmpty || event.content == '+');
}
