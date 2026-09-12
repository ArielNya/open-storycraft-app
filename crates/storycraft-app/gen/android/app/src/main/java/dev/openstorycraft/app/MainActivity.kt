package dev.openstorycraft.app

import android.os.Bundle
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
  }

  fun startJobService(skill: String) {
    JobForegroundService.start(this, skill)
  }

  fun stopJobService() {
    JobForegroundService.stop(this)
  }

  fun openAuthTab(url: String) {
    CustomTabs.open(this, url)
  }
}
