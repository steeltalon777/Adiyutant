package com.adiyutant.app.ui.common

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.adiyutant.app.R
import com.adiyutant.app.domain.model.Mood
import com.adiyutant.app.ui.theme.AccentGreen
import com.adiyutant.app.ui.theme.AccentRed
import com.adiyutant.app.ui.theme.Border
import com.adiyutant.app.ui.theme.Fg
import com.adiyutant.app.ui.theme.Muted
import com.adiyutant.app.ui.theme.Surface2

/**
 * Выбор самочувствия (App.jsx 993–1009): Плохо (AccentRed) / Нормально (Muted) /
 * Хорошо (AccentGreen); выбранный пункт — подсветка своим цветом.
 */
@Composable
fun MoodPicker(
    value: Mood,
    onChange: (Mood) -> Unit,
    modifier: Modifier = Modifier,
) {
    data class Option(val mood: Mood, val label: String, val color: Color)
    val options = listOf(
        Option(Mood.BAD, stringResource(R.string.mood_bad), AccentRed),
        Option(Mood.OK, stringResource(R.string.mood_ok), Muted),
        Option(Mood.GOOD, stringResource(R.string.mood_good), AccentGreen),
    )
    Row(
        modifier = modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        options.forEach { option ->
            val selected = value == option.mood
            val shape = RoundedCornerShape(10.dp)
            Box(
                modifier = Modifier
                    .weight(1f)
                    .clip(shape)
                    .background(if (selected) option.color.copy(alpha = 0.13f) else Surface2)
                    .border(1.dp, if (selected) option.color else Border, shape)
                    .clickable { onChange(option.mood) }
                    .padding(vertical = 10.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = option.label,
                    style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.SemiBold),
                    color = if (selected) Fg else Muted,
                    maxLines = 1,
                )
            }
        }
    }
}
