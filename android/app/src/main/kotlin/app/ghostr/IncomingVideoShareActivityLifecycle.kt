package app.ghostr

import android.content.Intent

internal data class IncomingVideoShareActivityDelivery(
    val intent: CapturedIncomingVideoIntent,
    val generation: Long,
)

internal class IncomingVideoShareActivityLifecycle(
    launchIntent: Intent,
    savedCaptureId: String?,
    private val intentCapture: IncomingVideoShareIntentCapture =
        IncomingVideoShareIntentCapture(),
) {
    private var capturedIntent = intentCapture.restore(launchIntent, savedCaptureId)
    private var shareGeneration = 0L

    val savedCaptureId: String?
        get() = capturedIntent.savedId

    fun configureEngine(): IncomingVideoShareActivityDelivery {
        return delivery(generationFor(capturedIntent.intent))
    }

    fun receive(intent: Intent): IncomingVideoShareActivityDelivery {
        capturedIntent = intentCapture.new(intent)
        return delivery(generationFor(intent))
    }

    fun acknowledge(
        generation: Long,
        launcherIntent: Intent = Intent(Intent.ACTION_MAIN),
    ): Intent? {
        if (!isCurrentShare(generation)) return null
        capturedIntent = intentCapture.new(launcherIntent)
        return launcherIntent
    }

    private fun delivery(generation: Long): IncomingVideoShareActivityDelivery {
        return IncomingVideoShareActivityDelivery(capturedIntent, generation)
    }

    private fun generationFor(intent: Intent): Long {
        if (intent.action == Intent.ACTION_SEND) shareGeneration += 1
        return shareGeneration
    }

    private fun isCurrentShare(generation: Long): Boolean {
        return generation == shareGeneration &&
            capturedIntent.intent.action == Intent.ACTION_SEND
    }
}
