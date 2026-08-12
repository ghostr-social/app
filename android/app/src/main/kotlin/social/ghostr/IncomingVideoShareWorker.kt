package social.ghostr

import java.util.concurrent.Executors

internal object IncomingVideoShareWorker {
    private val executor = Executors.newSingleThreadExecutor { operation ->
        Thread(operation, "incoming-video-share").apply { isDaemon = true }
    }

    fun execute(operation: () -> Unit) {
        executor.execute(operation)
    }
}
