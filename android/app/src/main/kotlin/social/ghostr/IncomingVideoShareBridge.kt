package social.ghostr

import android.content.Context
import android.content.Intent
import android.os.Handler
import android.os.Looper
import io.flutter.plugin.common.BinaryMessenger
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import java.util.ArrayDeque

internal class IncomingVideoShareBridge(
    context: Context,
    messenger: BinaryMessenger,
    initialIntent: CapturedIncomingVideoIntent,
    initialGeneration: Long,
    private val acknowledgeShare: (Long) -> Unit,
) {
    private val channel = MethodChannel(messenger, CHANNEL_NAME)
    private val mainHandler = Handler(Looper.getMainLooper())
    private val receiver = IncomingVideoShareReceiver(context)
    private val worker = IncomingVideoShareWorker
    private val stateLock = Any()
    private val pending = ArrayDeque<PendingDelivery>()
    private val deliveredGenerations = mutableMapOf<String, Long>()
    @Volatile private var latestGeneration = initialGeneration
    @Volatile private var disposed = false

    init {
        channel.setMethodCallHandler(::handleMethodCall)
        worker.execute { bootstrap(initialIntent.snapshot(), initialGeneration) }
    }
    fun receive(intent: CapturedIncomingVideoIntent, generation: Long) {
        latestGeneration = generation
        enqueue(intent, generation, notify = true)
    }
    fun dispose() {
        clearPendingDeliveries()
        channel.setMethodCallHandler(null)
    }
    private fun bootstrap(intent: CapturedIncomingVideoIntent, generation: Long) {
        receiver.deleteStaleCacheFiles()
        val restored = receiver.restorePending()
        if (restored != null) storeDelivery(PendingDelivery(restored, false, generation))
        if (!receiver.isHandled(intent) &&
            (restored == null || restored.sourceKey != intent.captureId)
        ) {
            process(intent, generation, notify = false)
        }
    }
    private fun handleMethodCall(call: MethodCall, result: MethodChannel.Result) {
        when (call.method) {
            TAKE_PENDING_VIDEO -> worker.execute { deliverPending(result) }
            ACKNOWLEDGE_VIDEO -> acknowledgeVideo(call.arguments, result)
            RELEASE_VIDEO -> releaseVideo(call.arguments, result)
            else -> result.notImplemented()
        }
    }
    private fun acknowledgeVideo(arguments: Any?, result: MethodChannel.Result) {
        val path = requiredPath(arguments, result) ?: return
        performOperation(result) {
            val generation = deliveredGeneration(path)
            receiver.acknowledge(path)
            synchronized(stateLock) { deliveredGenerations.remove(path) }
            generation
        }
    }
    private fun releaseVideo(arguments: Any?, result: MethodChannel.Result) {
        val path = requiredPath(arguments, result) ?: return
        performOperation(result) {
            val generation = deliveredGeneration(path)
            receiver.release(path)
            synchronized(stateLock) { deliveredGenerations.remove(path) }
            generation
        }
    }
    private fun deliveredGeneration(path: String): Long? {
        return synchronized(stateLock) { deliveredGenerations[path] }
    }
    private fun requiredPath(arguments: Any?, result: MethodChannel.Result): String? {
        val path = arguments as? String
        if (path != null) return path
        result.error(ERROR_CODE, ERROR_MESSAGE, null)
        return null
    }
    private fun enqueue(intent: CapturedIncomingVideoIntent, generation: Long, notify: Boolean) {
        worker.execute { process(intent.snapshot(), generation, notify) }
    }
    private fun process(intent: CapturedIncomingVideoIntent, generation: Long, notify: Boolean) {
        if (intent.intent.action != Intent.ACTION_SEND) return
        val delivery = try {
            PendingDelivery(receiver.receive(intent), false, generation)
        } catch (_: IncomingVideoShareException) {
            PendingDelivery(null, true, generation)
        }
        if (!storeDelivery(delivery)) {
            return
        }
        if (notify) notifyVideoAvailable()
    }
    private fun deliverPending(result: MethodChannel.Result) {
        val delivery = takeDelivery() ?: PendingDelivery(null, false, 0L)
        delivery.share?.let { share ->
            synchronized(stateLock) {
                deliveredGenerations[share.path] = delivery.generation
            }
        }
        mainHandler.post { completeDelivery(delivery, result) }
    }
    private fun completeDelivery(delivery: PendingDelivery, result: MethodChannel.Result) {
        if (disposed) {
            result.error(ERROR_CODE, ERROR_MESSAGE, null)
        } else if (delivery.failed) {
            result.error(ERROR_CODE, ERROR_MESSAGE, null)
            if (delivery.generation == latestGeneration) {
                acknowledgeShare(latestGeneration)
            }
        } else {
            result.success(delivery.share?.toMap())
        }
    }

    private fun storeDelivery(delivery: PendingDelivery): Boolean {
        val evicted = synchronized(stateLock) {
            if (disposed) return false
            pending.addLast(delivery)
            if (pending.size > MAX_PENDING_DELIVERIES) pending.removeFirst()
            else null
        }
        evicted?.let(::releaseDelivery)
        return true
    }

    private fun takeDelivery(): PendingDelivery? {
        return synchronized(stateLock) { pending.pollFirst() }
    }

    private fun clearPendingDeliveries() {
        synchronized(stateLock) {
            disposed = true
            pending.clear()
        }
    }

    private fun releaseDelivery(delivery: PendingDelivery) {
        delivery.share?.let { receiver.release(it.path) }
    }

    private fun performOperation(result: MethodChannel.Result, operation: () -> Long?) {
        worker.execute {
            try {
                val generation = operation()
                completeOperation(result) { acknowledgeGeneration(generation) }
            } catch (_: Exception) {
                completeError(result)
            }
        }
    }

    private fun acknowledgeGeneration(generation: Long?) {
        if (generation == latestGeneration) acknowledgeShare(latestGeneration)
    }

    private fun completeOperation(result: MethodChannel.Result, completion: () -> Unit) {
        mainHandler.post {
            completion()
            result.success(null)
        }
    }

    private fun completeError(result: MethodChannel.Result) {
        mainHandler.post { result.error(ERROR_CODE, ERROR_MESSAGE, null) }
    }

    private fun notifyVideoAvailable() {
        mainHandler.post {
            if (!disposed) channel.invokeMethod(VIDEO_AVAILABLE, null)
        }
    }

    private data class PendingDelivery(
        val share: IncomingVideoShare?,
        val failed: Boolean,
        val generation: Long,
    )

    private companion object {
        const val CHANNEL_NAME = "app.ghostr/incoming_video_share"
        const val TAKE_PENDING_VIDEO = "takePendingVideo"
        const val ACKNOWLEDGE_VIDEO = "acknowledgeVideo"
        const val RELEASE_VIDEO = "releaseVideo"
        const val VIDEO_AVAILABLE = "videoAvailable"
        const val ERROR_CODE = "incoming_video_unavailable"
        const val ERROR_MESSAGE = "Could not open the shared video."
        const val MAX_PENDING_DELIVERIES = 1
    }
}
