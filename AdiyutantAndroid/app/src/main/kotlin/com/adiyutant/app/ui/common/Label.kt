package com.adiyutant.app.ui.common

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.adiyutant.app.ui.theme.Muted2

/**
 * Заголовок секции (ТЗ 12.2): 12sp/600/Muted2/uppercase/tracking.
 * Паттерн из JSX Label (App.jsx 973–976): 11px/600, letter-spacing 0.04em.
 *
 * Compose UI 1.9 не содержит TextTransform (удалён из ui-text) — uppercase
 * применяется к строке напрямую (Label-тексты короткие, детерминированы).
 */
@Composable
fun Label(text: String, modifier: Modifier = Modifier) {
    Text(
        text = text.uppercase(),
        style = MaterialTheme.typography.labelSmall.copy(
            fontWeight = FontWeight.SemiBold,
            letterSpacing = 0.4.sp,
        ),
        color = Muted2,
        modifier = modifier.padding(bottom = 8.dp),
    )
}
