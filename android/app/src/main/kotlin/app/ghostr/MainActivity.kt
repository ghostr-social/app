package app.ghostr

import android.content.Intent
import android.os.Bundle
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine

class MainActivity : FlutterActivity() {
    private var incomingVideoShareBridge: IncomingVideoShareBridge? = null
    private lateinit var shareLifecycle: IncomingVideoShareActivityLifecycle

    override fun onCreate(savedInstanceState: Bundle?) {
        shareLifecycle = IncomingVideoShareActivityLifecycle(
            intent,
            savedInstanceState?.getString(CAPTURE_ID_STATE),
        )
        super.onCreate(savedInstanceState)
    }

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        incomingVideoShareBridge?.dispose()
        val delivery = shareLifecycle.configureEngine()
        incomingVideoShareBridge = IncomingVideoShareBridge(
            applicationContext,
            flutterEngine.dartExecutor.binaryMessenger,
            delivery.intent,
            delivery.generation,
            ::acknowledgeShare,
        )
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        deliver(shareLifecycle.receive(intent))
    }

    override fun onSaveInstanceState(outState: Bundle) {
        shareLifecycle.savedCaptureId?.let {
            outState.putString(CAPTURE_ID_STATE, it)
        }
        super.onSaveInstanceState(outState)
    }

    override fun onDestroy() {
        incomingVideoShareBridge?.dispose()
        incomingVideoShareBridge = null
        super.onDestroy()
    }

    private fun deliver(delivery: IncomingVideoShareActivityDelivery) {
        val intent = delivery.intent
        incomingVideoShareBridge?.receive(intent, delivery.generation)
    }

    private fun acknowledgeShare(generation: Long) {
        val launcherIntent = Intent(Intent.ACTION_MAIN)
        val acknowledged = shareLifecycle.acknowledge(generation, launcherIntent)
        acknowledged?.let(::setIntent)
    }

    private companion object {
        const val CAPTURE_ID_STATE = "incoming_video_capture_id"
    }
}
