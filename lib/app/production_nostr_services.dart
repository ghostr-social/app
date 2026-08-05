import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/features/publish/data/nostr_video_publisher.dart';
import 'package:ghostr/features/publish/domain/nostr_video_publisher_port.dart';
import 'package:ghostr/features/session/domain/nostr_session_port.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/social/domain/nostr_social_port.dart';
import 'package:ghostr/platform/nostr/build_ndk.dart';
import 'package:ghostr/platform/nostr/ndk_blossom_video_uploader.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_session.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:ghostr/platform/nostr/rust_broadcast_adapter.dart';
import 'package:ghostr/platform/nostr/rust_nostr_event_client.dart';
import 'package:ghostr/platform/nostr/rust_nostr_session.dart';
import 'package:ghostr/src/rust/api/session_control.dart' as engine;
import 'package:ndk/ndk.dart';

typedef ProductionNdkBuilder = Ndk Function();

ProductionNostrServices buildProductionNostrServices(
  AppSettings settings, {
  ProductionNdkBuilder ndkBuilder = buildNdk,
}) {
  final ndk = ndkBuilder();
  const broadcast = RustBroadcastAdapter();
  final eventClient = RustNostrEventClient(ndk: ndk, broadcast: broadcast);
  return ProductionNostrServices(
    ProductionNostrAdapters(
      RustNostrSession(
        local: NdkNostrSession(ndk),
        reset: (account) => engine.ffiResetNostrSession(
          expectedPublicKeyHex: account?.value,
        ),
      ),
      NdkNostrSocial(
        ndk: ndk,
        eventClient: eventClient,
        broadcast: broadcast,
      ),
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
    this.adapters,
    this.eventClient,
    this.publisher,
  );

  final ProductionNostrAdapters adapters;
  final NostrEventClient eventClient;
  final NostrVideoPublisherPort publisher;
}

class ProductionNostrAdapters {
  const ProductionNostrAdapters(this.session, this.social);

  final NostrSessionPort session;
  final NostrSocialPort social;
}
