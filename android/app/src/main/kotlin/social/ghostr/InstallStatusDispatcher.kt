package social.ghostr

import android.app.Activity
import android.content.Intent
import java.lang.ref.WeakReference

internal object InstallStatusDispatcher {
    private var activity = WeakReference<Activity>(null)
    private var sink: ((InstallStatus) -> Unit)? = null

    @Synchronized
    fun attach(activity: Activity, sink: (InstallStatus) -> Unit) {
        this.activity = WeakReference(activity)
        this.sink = sink
    }

    @Synchronized
    fun detach(activity: Activity) {
        if (this.activity.get() !== activity) return
        this.activity.clear()
        sink = null
    }

    @Synchronized
    fun publish(status: InstallStatus, confirmation: Intent?) {
        val current = availableActivity()
        if (current != null && confirmation != null) {
            current.startActivity(confirmation)
        }
        val listener = sink ?: return
        current?.runOnUiThread { listener(status) }
    }

    private fun availableActivity(): Activity? {
        val current = activity.get() ?: return null
        return if (!current.isFinishing && !current.isDestroyed) current else null
    }
}
