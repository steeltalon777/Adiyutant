package com.adiyutant.app.ui.shell

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.compose.currentBackStackEntryAsState
import com.adiyutant.app.R
import com.adiyutant.app.domain.model.PlannerEvent
import com.adiyutant.app.domain.model.SheetState
import com.adiyutant.app.ui.navigation.AppNavGraph
import com.adiyutant.app.ui.navigation.Destinations
import com.adiyutant.app.ui.planner.PlannerViewModel
import com.adiyutant.app.ui.theme.Bg
import com.adiyutant.app.ui.theme.Border
import com.adiyutant.app.ui.theme.Muted2
import com.adiyutant.app.ui.theme.Surface2

/**
 * Root shell of the app (ADR-0002, TZ раздел 5).
 *
 * Hoists единственный [PlannerViewModel] на Activity scope: Today и Plan
 * получают один и тот же экземпляр, переключение вкладок не сбрасывает
 * состояние и активную сессию. Общие листы ([SheetState]) рендерятся поверх
 * [AppNavGraph] здесь; в v1 это заглушка, контент приходит в Phase 3A.
 */
@Composable
fun AppShell(
    plannerViewModel: PlannerViewModel = viewModel(factory = PlannerViewModel.factory()),
) {
    val navController = androidx.navigation.compose.rememberNavController()
    val backStackEntry by navController.currentBackStackEntryAsState()
    val currentRoute = backStackEntry?.destination?.route

    val navigateToTab: (Destinations) -> Unit = { tab ->
        navController.navigate(tab.route) {
            popUpTo(navController.graph.startDestinationId) {
                saveState = true
            }
            launchSingleTop = true
            restoreState = true
        }
    }

    val state by plannerViewModel.state.collectAsState()
    val sheet = state.sheet

    // Системный back закрывает открытый лист без бизнес-мутаций (R5).
    BackHandler(enabled = sheet != null) {
        plannerViewModel.dispatch(PlannerEvent.CloseSheet)
    }

    Scaffold(
        containerColor = Bg,
        bottomBar = {
            BottomNavBar(
                currentRoute = currentRoute.orEmpty(),
                onTabSelected = navigateToTab,
            )
        },
    ) { innerPadding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .background(MaterialTheme.colorScheme.background),
        ) {
            AppNavGraph(
                navController = navController,
                planner = plannerViewModel,
                onNavigate = navigateToTab,
            )
            SheetHostPlaceholder(
                sheet = sheet,
                onClose = { plannerViewModel.dispatch(PlannerEvent.CloseSheet) },
            )
        }
    }
}

/**
 * Временный host общих листов (Phase 2): механика готова (scrim, поверх NavHost,
 * lossless close через [onClose]), контент листов придёт в Phase 3A
 * (ScheduleSheet, TaskDetailsSheet) и будет вынесен из этого файла.
 */
@Composable
private fun SheetHostPlaceholder(
    sheet: SheetState?,
    onClose: () -> Unit,
) {
    if (sheet == null) return
    Box(modifier = Modifier.fillMaxSize()) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(Color.Black.copy(alpha = 0.55f))
                .clickable(onClick = onClose),
        )
        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .align(androidx.compose.ui.Alignment.BottomCenter)
                .navigationBarsPadding(),
            shape = RoundedCornerShape(topStart = 18.dp, topEnd = 18.dp),
            color = Surface2,
            border = androidx.compose.foundation.BorderStroke(1.dp, Border),
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(24.dp),
            ) {
                Text(
                    text = stringResource(R.string.sheet_host_placeholder),
                    style = MaterialTheme.typography.bodyMedium,
                    color = Muted2,
                )
            }
        }
    }
}
