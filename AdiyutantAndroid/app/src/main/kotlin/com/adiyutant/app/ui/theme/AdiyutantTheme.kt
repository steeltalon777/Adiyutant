package com.adiyutant.app.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

// ─── Adiyutant UI Kit — Material 3 as infrastructure ─────────────────────
// Material 3 components (Scaffold, NavigationBar, Cards) are used for their
// behavior and accessibility structure. All colors come from the Adiyutant
// dark tokens above, NOT from Material's default palettes (ADR-0002).

private val DarkColorScheme = darkColorScheme(
    primary = Accent,
    onPrimary = Fg,
    primaryContainer = AccentSoft,
    onPrimaryContainer = Fg,
    secondary = AccentGreen,
    onSecondary = BgDeep,
    secondaryContainer = AccentGreenDeep,
    onSecondaryContainer = Fg,
    tertiary = AccentPurple,
    onTertiary = Fg,
    tertiaryContainer = AccentPurpleDeep,
    onTertiaryContainer = Fg,
    background = Bg,
    onBackground = Fg,
    surface = Surface,
    onSurface = Fg,
    surfaceVariant = Surface2,
    onSurfaceVariant = Muted,
    outline = Border,
    error = Color(0xFFFF6B6B),
    onError = BgDeep,
)

@Composable
fun AdiyutantTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = DarkColorScheme,
        typography = AdiyutantTypography,
        shapes = AdiyutantShapes,
        content = content,
    )
}
