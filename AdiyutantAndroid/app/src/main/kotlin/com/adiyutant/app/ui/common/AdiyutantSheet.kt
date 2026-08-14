package com.adiyutant.app.ui.common

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import com.adiyutant.app.ui.theme.Border
import com.adiyutant.app.ui.theme.Fg
import com.adiyutant.app.ui.theme.Surface

/**
 * Общий scaffold bottom sheet (ТЗ 12.4, 13.1).
 *
 * Подложка `rgba(0,0,0,0.55)` (клик — lossless close), скругление верха 18dp,
 * handle 36×4 [Border], maxHeight ~85%, вертикальный скролл, [BackHandler] →
 * lossless close, insets `navigationBarsPadding`. Закрытие никогда не меняет
 * бизнес-данные: [onClose] — единственный выход (reducer CloseSheet/cancel).
 */
@Composable
fun AdiyutantSheet(
    title: String?,
    onClose: () -> Unit,
    modifier: Modifier = Modifier,
    content: @Composable ColumnScope.() -> Unit,
) {
    Box(modifier = modifier.fillMaxSize()) {
        // Подложка: клик по ней — закрытие (lossless).
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(Color.Black.copy(alpha = 0.55f))
                .clickable(onClick = onClose),
        )
        // Системный back закрывает лист без бизнес-мутаций (R5).
        BackHandler(onBack = onClose)

        BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
            Surface(
                modifier = Modifier
                    .fillMaxWidth()
                    .align(Alignment.BottomCenter)
                    .heightIn(max = maxHeight * 0.85f)
                    .navigationBarsPadding(),
                shape = RoundedCornerShape(topStart = 18.dp, topEnd = 18.dp),
                color = Surface,
                border = BorderStroke(1.dp, Border),
            ) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .verticalScroll(rememberScrollState())
                        .padding(bottom = 24.dp),
                ) {
                    // Handle 36×4.
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(vertical = 8.dp),
                        horizontalArrangement = Arrangement.Center,
                    ) {
                        Box(
                            modifier = Modifier
                                .width(36.dp)
                                .height(4.dp)
                                .background(Border, RoundedCornerShape(2.dp)),
                        )
                    }
                    if (title != null) {
                        Text(
                            text = title,
                            style = MaterialTheme.typography.titleMedium,
                            color = Fg,
                            modifier = Modifier.padding(horizontal = 20.dp, vertical = 6.dp),
                        )
                    }
                    content()
                }
            }
        }
    }
}
