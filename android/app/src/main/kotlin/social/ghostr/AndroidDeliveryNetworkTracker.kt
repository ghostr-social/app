package social.ghostr

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.Build
import java.util.concurrent.atomic.AtomicLong

private val networkGenerations = AtomicLong(0)

internal data class DeliveryNetworkStatus(
    val networkClass: String,
    val generation: Long,
) {
    fun payload(): Map<String, Any> = mapOf(
        "class" to networkClass,
        "generation" to generation,
    )
}

internal class AndroidDeliveryNetworkTracker(
    context: Context,
    private val publish: (DeliveryNetworkStatus) -> Unit,
) {
    private val manager = context.getSystemService(ConnectivityManager::class.java)
    private val callback = object : ConnectivityManager.NetworkCallback() {
        override fun onAvailable(network: Network) = updateFromActiveNetwork()
        override fun onLost(network: Network) = updateFromActiveNetwork()
        override fun onUnavailable() = updateFromActiveNetwork()

        override fun onCapabilitiesChanged(
            network: Network,
            capabilities: NetworkCapabilities,
        ) = updateFromActiveNetwork()
    }
    private var current = status(networkClass(manager))
    private var disposed = false

    init {
        register()
    }

    @Synchronized
    fun snapshot(): DeliveryNetworkStatus = current

    @Synchronized
    fun dispose() {
        if (disposed) return
        disposed = true
        runCatching { manager.unregisterNetworkCallback(callback) }
    }

    private fun register() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            manager.registerDefaultNetworkCallback(callback)
            return
        }
        val request = NetworkRequest.Builder()
            .addCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
            .build()
        manager.registerNetworkCallback(request, callback)
    }

    private fun updateFromActiveNetwork() {
        val next = synchronized(this) {
            if (disposed) return
            val value = networkClass(manager)
            if (value == current.networkClass) return
            current = status(value)
            current
        }
        publish(next)
    }
}

private fun networkClass(manager: ConnectivityManager): String {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.M) {
        return legacyNetworkClass(manager)
    }
    val network = manager.activeNetwork ?: return "unavailable"
    val capabilities = manager.getNetworkCapabilities(network) ?: return "unavailable"
    if (!capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)) {
        return "unavailable"
    }
    if (isConstrained(manager, capabilities)) return "constrained"
    return when {
        capabilities.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) -> "wifi"
        capabilities.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) -> "cellular"
        capabilities.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) -> "wired"
        else -> "unavailable"
    }
}

private fun isConstrained(
    manager: ConnectivityManager,
    capabilities: NetworkCapabilities,
): Boolean {
    if (!capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_RESTRICTED)) return true
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M &&
        !capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED)) return true
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P &&
        !capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_ROAMING)) return true
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N &&
        manager.restrictBackgroundStatus ==
        ConnectivityManager.RESTRICT_BACKGROUND_STATUS_ENABLED) return true
    return capabilities.hasTransport(NetworkCapabilities.TRANSPORT_VPN)
}

private fun status(networkClass: String): DeliveryNetworkStatus {
    return DeliveryNetworkStatus(networkClass, networkGenerations.incrementAndGet())
}

@Suppress("DEPRECATION")
private fun legacyNetworkClass(manager: ConnectivityManager): String {
    val info = manager.activeNetworkInfo ?: return "unavailable"
    if (!info.isConnected) return "unavailable"
    return when (info.type) {
        ConnectivityManager.TYPE_WIFI -> "wifi"
        ConnectivityManager.TYPE_MOBILE -> "cellular"
        ConnectivityManager.TYPE_ETHERNET -> "wired"
        else -> "unavailable"
    }
}
