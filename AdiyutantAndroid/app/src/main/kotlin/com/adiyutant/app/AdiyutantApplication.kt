package com.adiyutant.app

import android.app.Application
import com.adiyutant.app.di.ServiceLocator

class AdiyutantApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        // Manual DI container initialized once at process start (ADR-0002).
        ServiceLocator
    }
}
