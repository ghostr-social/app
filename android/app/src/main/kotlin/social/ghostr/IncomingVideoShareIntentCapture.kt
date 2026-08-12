package social.ghostr

import android.content.Intent
import java.util.UUID

@JvmInline
internal value class IncomingVideoShareCaptureId(val value: String)

internal data class CapturedIncomingVideoIntent(
    val intent: Intent,
    val captureId: IncomingVideoShareCaptureId?,
) {
    val savedId: String?
        get() = captureId?.value

    fun snapshot(): CapturedIncomingVideoIntent = copy(intent = Intent(intent))
}

internal class IncomingVideoShareIntentCapture(
    private val nextId: () -> String = { UUID.randomUUID().toString() },
) {
    fun new(intent: Intent): CapturedIncomingVideoIntent = captured(intent, null)

    fun restore(intent: Intent, savedId: String?): CapturedIncomingVideoIntent {
        return captured(intent, savedId?.takeIf(String::isNotBlank))
    }

    private fun captured(intent: Intent, restoredId: String?): CapturedIncomingVideoIntent {
        val captureId = if (intent.action == Intent.ACTION_SEND) {
            IncomingVideoShareCaptureId(restoredId ?: nextId())
        } else null
        return CapturedIncomingVideoIntent(Intent(intent), captureId)
    }
}
