package com.adiyutant.app.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

// ─── Adiyutant UI Kit — typography roles ─────────────────────────────────
// Sources: docs/design/today-screen.html (fs-h1 26px, fs-h2 17px, fs-h3 14px,
// fs-body 14px, fs-meta 11px; display/body/mono font stacks).
//
// Font stacks from the design map onto system fonts on Android:
//   --font-display / --font-body → SansSerif
//   --font-mono                   → Monospace

private val displayStack = FontFamily.SansSerif
private val monoStack = FontFamily.Monospace

val AdiyutantTypography = Typography(
    displayLarge = TextStyle(
        fontFamily = displayStack,
        fontSize = 26.sp,
        fontWeight = FontWeight.Bold,
        letterSpacing = (-0.6).sp,
        lineHeight = 27.sp,
    ),
    titleLarge = TextStyle(
        fontFamily = displayStack,
        fontSize = 22.sp,
        fontWeight = FontWeight.Bold,
        letterSpacing = (-0.4).sp,
        lineHeight = 24.sp,
    ),
    titleMedium = TextStyle(
        fontFamily = displayStack,
        fontSize = 17.sp,
        fontWeight = FontWeight.SemiBold,
        letterSpacing = (-0.2).sp,
    ),
    titleSmall = TextStyle(
        fontFamily = displayStack,
        fontSize = 14.sp,
        fontWeight = FontWeight.SemiBold,
        letterSpacing = (-0.1).sp,
    ),
    bodyLarge = TextStyle(
        fontFamily = FontFamily.SansSerif,
        fontSize = 14.sp,
        fontWeight = FontWeight.Normal,
        lineHeight = 20.sp,
    ),
    bodyMedium = TextStyle(
        fontFamily = FontFamily.SansSerif,
        fontSize = 13.sp,
        fontWeight = FontWeight.Medium,
        lineHeight = 17.sp,
    ),
    bodySmall = TextStyle(
        fontFamily = FontFamily.SansSerif,
        fontSize = 12.sp,
        fontWeight = FontWeight.Normal,
        lineHeight = 15.sp,
    ),
    labelSmall = TextStyle(
        fontFamily = FontFamily.SansSerif,
        fontSize = 11.sp,
        fontWeight = FontWeight.Medium,
        lineHeight = 13.sp,
    ),
    labelMedium = TextStyle(
        fontFamily = monoStack,
        fontSize = 11.5.sp,
        fontWeight = FontWeight.SemiBold,
        letterSpacing = (-0.2).sp,
    ),
)
