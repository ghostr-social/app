package social.ghostr

import android.app.Activity
import io.flutter.plugin.common.BinaryMessenger
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel

internal class AndroidDeliveryNetworkBridge(
    private val activity: Activity,
    messenger: BinaryMessenger,
) : MethodChannel.MethodCallHandler {
    private val channel = MethodChannel(messenger, CHANNEL_NAME)
    private val tracker = AndroidDeliveryNetworkTracker(
        activity.applicationContext,
        ::publish,
    )
    @Volatile
    private var disposed = false

    init {
        channel.setMethodCallHandler(this)
    }

    override fun onMethodCall(call: MethodCall, result: MethodChannel.Result) {
        if (call.method == "readNetworkStatus") {
            result.success(tracker.snapshot().payload())
        } else {
            result.notImplemented()
        }
    }

    fun dispose() {
        disposed = true
        tracker.dispose()
        channel.setMethodCallHandler(null)
    }

    private fun publish(status: DeliveryNetworkStatus) {
        activity.runOnUiThread {
            if (!disposed) channel.invokeMethod("networkStatusChanged", status.payload())
        }
    }

    private companion object {
        const val CHANNEL_NAME = "social.ghostr/network/v1"
    }
}
