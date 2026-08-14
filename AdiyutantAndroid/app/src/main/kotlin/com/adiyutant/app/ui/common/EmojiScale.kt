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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.adiyutant.app.ui.theme.Accent
import com.adiyutant.app.ui.theme.Border
import com.adiyutant.app.ui.theme.Fg
import com.adiyutant.app.ui.theme.Muted
import com.adiyutant.app.ui.theme.Surface2

/**
 * Шкала выбора 0..N (App.jsx 978–991): кнопки flex-ряд, выбранная —
 * accent-подсветка, border Accent, текст Fg; остальные Surface2 + Muted.
 * Используется для энергии (0..3) и фокуса (0..3) в чек-ине и EOD.
 */
@Composable
fun EmojiScale(
    value: Int,
    onChange: (Int) -> Unit,
    labels: List<String>,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        labels.forEachIndexed { index, label ->
            val selected = index == value
            val shape = RoundedCornerShape(10.dp)
            Box(
                modifier = Modifier
                    .weight(1f)
                    .clip(shape)
                    .background(if (selected) Accent.copy(alpha = 0.13f) else Surface2)
                    .border(1.dp, if (selected) Accent else Border, shape)
                    .clickable { onChange(index) }
                    .padding(vertical = 10.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = label,
                    style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.SemiBold),
                    color = if (selected) Fg else Muted,
                    maxLines = 1,
                )
            }
        }
    }
}
