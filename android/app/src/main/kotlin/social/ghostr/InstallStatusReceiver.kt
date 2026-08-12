package social.ghostr

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.PackageInstaller
import android.os.Build

class InstallStatusReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val status = parseStatus(intent)
        InstallStatusStore(context).write(status)
        InstallStatusDispatcher.publish(status, confirmationIntent(intent))
    }

    private fun parseStatus(intent: Intent): InstallStatus {
        val sessionId = intent.getIntExtra(PackageInstaller.EXTRA_SESSION_ID, -1)
        val code = intent.getIntExtra(
            PackageInstaller.EXTRA_STATUS,
            PackageInstaller.STATUS_FAILURE,
        )
        return when (code) {
            PackageInstaller.STATUS_PENDING_USER_ACTION -> pendingStatus(sessionId)
            PackageInstaller.STATUS_SUCCESS -> InstallStatus(sessionId, "succeeded")
            else -> failureStatus(intent, sessionId, code)
        }
    }

    private fun pendingStatus(sessionId: Int): InstallStatus {
        return InstallStatus(
            sessionId,
            "pendingUserAction",
            "Confirm this update with Android.",
        )
    }

    private fun failureStatus(intent: Intent, sessionId: Int, code: Int): InstallStatus {
        val message = intent.getStringExtra(PackageInstaller.EXTRA_STATUS_MESSAGE)
            ?: "Android could not install the update."
        return InstallStatus(sessionId, "failed", message, code)
    }

    @Suppress("DEPRECATION")
    private fun confirmationIntent(status: Intent): Intent? {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            status.getParcelableExtra(Intent.EXTRA_INTENT, Intent::class.java)
        } else {
            status.getParcelableExtra(Intent.EXTRA_INTENT)
        }
    }
}
