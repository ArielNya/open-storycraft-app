package dev.openstorycraft.app

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat

/**
 * Keeps the process alive while a skill job streams to a preview file.
 * Android will kill a silent WebView; this dataSync FGS is the contract.
 */
class JobForegroundService : Service() {
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val skill = intent?.getStringExtra(EXTRA_SKILL) ?: "skill"
        ensureChannel()
        val notification =
            NotificationCompat.Builder(this, CHANNEL_ID)
                .setContentTitle("Open Storycraft")
                .setContentText("Generating $skill")
                .setSmallIcon(R.mipmap.ic_launcher)
                .setOngoing(true)
                .setSilent(true)
                .build()
        if (Build.VERSION.SDK_INT >= 29) {
            startForeground(
                NOTIFICATION_ID,
                notification,
                ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC,
            )
        } else {
            @Suppress("DEPRECATION")
            startForeground(NOTIFICATION_ID, notification)
        }
        return START_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private fun ensureChannel() {
        if (Build.VERSION.SDK_INT < 26) return
        val mgr = getSystemService(NotificationManager::class.java) ?: return
        mgr.createNotificationChannel(
            NotificationChannel(CHANNEL_ID, "Storycraft jobs", NotificationManager.IMPORTANCE_LOW),
        )
    }

    companion object {
        const val CHANNEL_ID = "storycraft-jobs"
        const val NOTIFICATION_ID = 42
        const val EXTRA_SKILL = "skill"

        fun start(context: Context, skill: String) {
            val intent = Intent(context, JobForegroundService::class.java)
            intent.putExtra(EXTRA_SKILL, skill)
            if (Build.VERSION.SDK_INT >= 26) {
                context.startForegroundService(intent)
            } else {
                context.startService(intent)
            }
        }

        fun stop(context: Context) {
            context.stopService(Intent(context, JobForegroundService::class.java))
        }
    }
}
