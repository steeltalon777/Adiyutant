package com.adiyutant.app.ui.common

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.adiyutant.app.ui.theme.Border
import com.adiyutant.app.ui.theme.Fg
import com.adiyutant.app.ui.theme.Muted2
import com.adiyutant.app.ui.theme.Surface

/**
 * Мини-карточка KPI (App.jsx 1634–1642): label 10sp uppercase Muted2,
 * value 14/600 цветом [accent]. Используется в EndDaySheet
 * (Сделано — green, Заметок — blue, Факт-сессий — purple).
 */
@Composable
fun KpiMini(
    label: String,
    value: String,
    accent: Color = Fg,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(10.dp))
            .background(Surface)
            .border(1.dp, Border, RoundedCornerShape(10.dp))
            .padding(horizontal = 10.dp, vertical = 8.dp),
    ) {
        Text(
            text = label.uppercase(),
            style = MaterialTheme.typography.labelSmall.copy(
                fontSize = 10.sp,
                fontWeight = FontWeight.SemiBold,
                letterSpacing = 0.4.sp,
            ),
            color = Muted2,
            maxLines = 1,
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodyMedium.copy(
                fontSize = 14.sp,
                fontWeight = FontWeight.SemiBold,
            ),
            color = accent,
            modifier = Modifier.padding(top = 2.dp),
        )
    }
}
