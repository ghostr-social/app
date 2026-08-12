package app.ghostr

import android.content.Context
import android.content.ContentProvider
import android.content.Intent
import android.content.pm.ProviderInfo
import android.net.Uri
import org.robolectric.RuntimeEnvironment
import org.robolectric.shadows.ShadowContentResolver

internal const val SHARE_AUTHORITY = "app.ghostr.test.share"
internal val SHARE_URI: Uri = Uri.parse("content://$SHARE_AUTHORITY/video")

internal fun shareTestContext(provider: ContentProvider): Context {
    val context = RuntimeEnvironment.getApplication()
    provider.attachInfo(
        context,
        ProviderInfo().apply {
            authority = SHARE_AUTHORITY
            exported = true
            grantUriPermissions = true
        },
    )
    ShadowContentResolver.registerProviderInternal(SHARE_AUTHORITY, provider)
    return context
}

internal fun shareIntent(type: String? = "video/*", uri: Uri = SHARE_URI): Intent {
    return Intent(Intent.ACTION_SEND).setType(type).putExtra(Intent.EXTRA_STREAM, uri)
}

internal fun capturedShareIntent(
    type: String? = "video/*",
    uri: Uri = SHARE_URI,
    id: String = "test-capture",
): CapturedIncomingVideoIntent {
    return CapturedIncomingVideoIntent(
        shareIntent(type, uri),
        IncomingVideoShareCaptureId(id),
    )
}
