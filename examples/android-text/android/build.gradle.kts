plugins {
    id("com.android.application") version "8.11.0"
    id("org.jetbrains.kotlin.android") version "2.0.21"
}

android {
    namespace = "dev.rune.android_text"
    compileSdk = 35
    defaultConfig {
        applicationId = "dev.rune.android_text"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "0.0.1"
        ndk { abiFilters += "arm64-v8a" }
    }
    sourceSets["main"].apply {
        // Rune's Kotlin host, shared by any app that types into Rune.
        java.srcDir("../../../crates/rune/android/kotlin")
        // libandroid_text.so, staged by scripts/android-apk.sh from the cargo build.
        jniLibs.srcDir(providers.gradleProperty("rune.jniLibs").getOrElse("build/rune-jni"))
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

kotlin { jvmToolchain(17) }
