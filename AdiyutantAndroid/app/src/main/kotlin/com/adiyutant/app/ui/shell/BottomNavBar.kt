package com.adiyutant.app.ui.shell

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.adiyutant.app.ui.navigation.Destinations
import com.adiyutant.app.ui.theme.Bg
import com.adiyutant.app.ui.theme.Muted2
import com.adiyutant.app.ui.theme.Accent

/**
 * Bottom navigation bar mirroring the design's 5-tab tabbar.
 * Visual style follows the Adiyutant UI Kit, not Material defaults.
 */
@Composable
fun BottomNavBar(
    currentRoute: String,
    onTabSelected: (Destinations) -> Unit,
) {
    NavigationBar(
        containerColor = Bg,
        tonalElevation = 0.dp,
        modifier = Modifier
            .fillMaxWidth()
            .background(Bg),
    ) {
        Destinations.entries.forEach { tab ->
            NavigationBarItem(
                selected = currentRoute == tab.route,
                onClick = { onTabSelected(tab) },
                icon = {
                    Icon(
                        imageVector = TabIcon.forDestination(tab),
                        contentDescription = stringResource(tab.labelRes),
                    )
                },
                label = {
                    Text(
                        text = stringResource(tab.labelRes),
                        style = MaterialTheme.typography.labelSmall,
                    )
                },
                colors = androidx.compose.material3.NavigationBarItemDefaults.colors(
                    selectedIconColor = Accent,
                    selectedTextColor = Accent,
                    unselectedIconColor = Muted2,
                    unselectedTextColor = Muted2,
                    indicatorColor = androidx.compose.ui.graphics.Color.Transparent,
                ),
            )
        }
    }
}
