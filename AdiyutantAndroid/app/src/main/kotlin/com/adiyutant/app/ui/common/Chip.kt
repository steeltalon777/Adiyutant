package com.adiyutant.app.ui.common

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.adiyutant.app.ui.theme.Accent
import com.adiyutant.app.ui.theme.Border
import com.adiyutant.app.ui.theme.Fg
import com.adiyutant.app.ui.theme.Muted
import com.adiyutant.app.ui.theme.Muted2
import com.adiyutant.app.ui.theme.Surface2

/** Тон чипа: нейтральный или accent (слоты времени). */
enum class ChipTone { DEFAULT, ACCENT }

/**
 * Чип (ТЗ 12.4): pill ~28dp, для времени/статусов/слотов.
 *
 * [onClick] == null или [enabled] == false — некликабельный (disabled: Surface2+Muted2).
 * Выбранный: accent-рамка/подсветка. [ChipTone.ACCENT] — всегда accent-тон
 * (слоты ScheduleSheet: фон accent, текст accent, App.jsx 1933–1938).
 */
@Composable
fun Chip(
    text: String,
    onClick: (() -> Unit)?,
    selected: Boolean = false,
    tone: ChipTone = ChipTone.DEFAULT,
    enabled: Boolean = true,
    modifier: Modifier = Modifier,
) {
    val background = when {
        !enabled -> Surface2
        selected -> if (tone == ChipTone.ACCENT) Accent.copy(alpha = 0.18f) else Accent.copy(alpha = 0.13f)
        tone == ChipTone.ACCENT -> Accent.copy(alpha = 0.09f)
        else -> Surface2
    }
    val borderColor = when {
        !enabled -> Border
        selected -> if (tone == ChipTone.ACCENT) Accent.copy(alpha = 0.25f) else Accent
        tone == ChipTone.ACCENT -> Accent.copy(alpha = 0.25f)
        else -> Border
    }
    val textColor = when {
        !enabled -> Muted2
        selected -> if (tone == ChipTone.ACCENT) Accent else Fg
        tone == ChipTone.ACCENT -> Accent
        else -> Muted
    }
    val shape = RoundedCornerShape(999.dp)
    Box(
        modifier = modifier
            .clip(shape)
            .background(background)
            .border(1.dp, borderColor, shape)
            .then(
                if (onClick != null && enabled) Modifier.clickable(onClick = onClick) else Modifier,
            )
            .padding(horizontal = 10.dp, vertical = 6.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = text,
            style = MaterialTheme.typography.bodySmall.copy(fontWeight = FontWeight.SemiBold),
            color = textColor,
            maxLines = 1,
        )
    }
}
