package com.adiyutant.app.ui.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Shapes
import androidx.compose.ui.unit.dp

// ─── Adiyutant UI Kit — shape tokens ─────────────────────────────────────
// Sources: docs/design/today-screen.html (--radius-card 16, --radius-card-sm 12,
// --radius-btn 10).

val AdiyutantShapes = Shapes(
    extraSmall = RoundedCornerShape(8.dp),
    small = RoundedCornerShape(10.dp),
    medium = RoundedCornerShape(12.dp),
    large = RoundedCornerShape(16.dp),
    extraLarge = RoundedCornerShape(24.dp),
)
