package com.adiyutant.app.ui.shell

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.compose.currentBackStackEntryAsState
import com.adiyutant.app.ui.navigation.AppNavGraph
import com.adiyutant.app.ui.navigation.Destinations
import com.adiyutant.app.ui.theme.Bg

/**
 * Root shell of the app: bottom navigation bar + content host (ADR-0002).
 * Content renders through [AppNavGraph]; the bar highlights the active tab.
 */
@Composable
fun AppShell() {
    val navController = androidx.navigation.compose.rememberNavController()
    val backStackEntry by navController.currentBackStackEntryAsState()
    val currentRoute = backStackEntry?.destination?.route

    Scaffold(
        containerColor = Bg,
        bottomBar = {
            BottomNavBar(
                currentRoute = currentRoute.orEmpty(),
                onTabSelected = { tab ->
                    navController.navigate(tab.route) {
                        popUpTo(navController.graph.startDestinationId) {
                            saveState = true
                        }
                        launchSingleTop = true
                        restoreState = true
                    }
                },
            )
        },
    ) { innerPadding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .background(MaterialTheme.colorScheme.background),
        ) {
            AppNavGraph(navController = navController)
        }
    }
}
