package dev.openstorycraft.app

import android.graphics.Color
import android.os.Bundle
import android.view.View
import androidx.activity.SystemBarStyle
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    // The UI is always dark, so the bar icons are always light. The default
    // follows the system theme and draws dark icons over the dark header.
    enableEdgeToEdge(
      statusBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
      navigationBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
    )
    super.onCreate(savedInstanceState)

    // Edge-to-edge draws the webview under the status bar, the gesture bar and
    // the keyboard, and the WebView does not report those as safe-area insets.
    // Pad the content view instead: the header stays below the clock, the nav
    // above the gesture bar, and a focused field above the keyboard. The
    // window background (themes.xml) fills the padding in the app's colour.
    val content = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(content) { view, insets ->
      val edges = insets.getInsets(
        WindowInsetsCompat.Type.systemBars() or
          WindowInsetsCompat.Type.displayCutout() or
          WindowInsetsCompat.Type.ime()
      )
      view.setPadding(edges.left, edges.top, edges.right, edges.bottom)
      WindowInsetsCompat.CONSUMED
    }
  }
}
