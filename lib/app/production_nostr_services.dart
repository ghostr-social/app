import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/features/publish/data/nostr_video_publisher.dart';
import 'package:ghostr/features/publish/domain/nostr_video_publisher_port.dart';
import 'package:ghostr/features/session/domain/nostr_session_port.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/social/domain/nostr_social_port.dart';
import 'package:ghostr/platform/nostr/build_ndk.dart';
import 'package:ghostr/platform/nostr/ndk_blossom_video_uploader.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_client.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_session.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:ndk/ndk.dart';

typedef ProductionNdkBuilder = Ndk Function(List<RelayUrl> relays);

ProductionNostrServices buildProductionNostrServices(
  AppSettings settings, {
  ProductionNdkBuilder ndkBuilder = buildNdk,
}) {
  final ndk = ndkBuilder(settings.relays);
  final eventClient = NdkNostrEventClient(ndk: ndk, relays: settings.relays);
  return ProductionNostrServices(
    ndk,
    ProductionNostrAdapters(
      NdkNostrSession(ndk),
      NdkNostrSocial(ndk: ndk, relays: settings.relays),
    ),
    eventClient,
    NostrVideoPublisher(
      eventClient: eventClient,
      mediaUploader: NdkBlossomVideoUploader(
        ndk: ndk,
        servers: settings.blossomServers,
      ),
    ),
  );
}

class ProductionNostrServices {
  const ProductionNostrServices(
    this.ndk,
    this.adapters,
    this.eventClient,
    this.publisher,
  );

  final Ndk ndk;
  final ProductionNostrAdapters adapters;
  final NostrEventClient eventClient;
  final NostrVideoPublisherPort publisher;
}

class ProductionNostrAdapters {
  const ProductionNostrAdapters(this.session, this.social);

  final NostrSessionPort session;
  final NostrSocialPort social;
}
