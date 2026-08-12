package social.ghostr

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.net.Uri
import android.os.Build
import android.provider.Settings

internal class AndroidAppInspector(
    private val activity: Activity,
) {
    private val context: Context = activity.applicationContext

    fun installedApp(): Map<String, Any> {
        val info = context.packageManager.getPackageInfo(context.packageName, 0)
        return mapOf(
            "packageName" to context.packageName,
            "versionCode" to info.versionCodeLong(),
            "versionName" to (info.versionName ?: ""),
            "sdkInt" to Build.VERSION.SDK_INT,
            "supportedAbis" to Build.SUPPORTED_ABIS.toList(),
        )
    }

    fun networkAccess(): String {
        val manager = context.getSystemService(ConnectivityManager::class.java)
        val network = manager.activeNetwork ?: return "none"
        val capabilities = manager.getNetworkCapabilities(network) ?: return "none"
        return if (capabilities.hasTransport(NetworkCapabilities.TRANSPORT_WIFI)) {
            "wifi"
        } else {
            "other"
        }
    }

    fun canRequestInstalls(): Boolean {
        return Build.VERSION.SDK_INT < Build.VERSION_CODES.O ||
            context.packageManager.canRequestPackageInstalls()
    }

    fun openInstallPermissionSettings() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
        val intent = Intent(
            Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES,
            Uri.parse("package:${context.packageName}"),
        )
        activity.startActivity(intent)
    }
}

@Suppress("DEPRECATION")
private fun android.content.pm.PackageInfo.versionCodeLong(): Long {
    return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
        longVersionCode
    } else {
        versionCode.toLong()
    }
}
