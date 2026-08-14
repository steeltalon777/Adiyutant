package com.adiyutant.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import com.adiyutant.app.ui.shell.AppShell
import com.adiyutant.app.ui.theme.AdiyutantTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            AdiyutantTheme {
                AppShell()
            }
        }
    }
}
