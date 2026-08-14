package com.adiyutant.app.ui.feature.today

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import com.adiyutant.app.core.CorePort
import com.adiyutant.app.core.model.CurrentActivity
import com.adiyutant.app.core.model.TodaySnapshot
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class TodayUiState(
    val today: TodaySnapshot? = null,
    val activity: CurrentActivity? = null,
    val loading: Boolean = true,
)

/**
 * Hosts the "Сегодня" tab data. Reads through [CorePort] only — the screen
 * never touches the data source directly (ADR-0002).
 */
class TodayViewModel(
    private val corePort: CorePort,
) : ViewModel() {

    private val _uiState = MutableStateFlow(TodayUiState())
    val uiState: StateFlow<TodayUiState> = _uiState.asStateFlow()

    init {
        load()
    }

    fun load() {
        viewModelScope.launch {
            _uiState.value = TodayUiState(loading = true)
            val today = corePort.getTodaySnapshot()
            val activity = corePort.getCurrentActivity()
            _uiState.value = TodayUiState(
                today = today,
                activity = activity,
                loading = false,
            )
        }
    }

    companion object {
        fun factory(corePort: CorePort): ViewModelProvider.Factory =
            object : ViewModelProvider.Factory {
                @Suppress("UNCHECKED_CAST")
                override fun <T : ViewModel> create(modelClass: Class<T>): T =
                    TodayViewModel(corePort) as T
            }
    }
}
